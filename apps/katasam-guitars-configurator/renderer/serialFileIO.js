// serialFileIO.js
// Robust serial file read/write for BGG Windows App
// 
// ENHANCEMENTS IMPLEMENTED:
// ✅ Enhanced timeout system: 8s per KB + 60s bonus for files >10KB
// ✅ Line-by-line transmission: 75ms delays for large files, 50ms for smaller files  
// ✅ Refined error detection: Ignores "ERROR:" in echoed code content
// ✅ Large file handling: Dynamic timeouts and optimized transmission
// ✅ BOOTSEL interference prevention: Integrated with multiDeviceManager

const DEFAULT_TIMEOUT = 30000; // Enhanced default timeout (30s) - Increased timeout for file operations
const TIMEOUT_PER_KB = 8000; // Additional 8 seconds per KB for large files (enhanced from 5s)
const LARGE_FILE_BONUS = 60000; // Extra 60 seconds for files larger than 10KB

function writeToPort(port, chunk) {
  return new Promise((resolve, reject) => {
    port.write(chunk, (err) => err ? reject(err) : resolve());
  });
}

function drainPort(port) {
  return new Promise((resolve, reject) => {
    if (typeof port.drain !== 'function') {
      resolve();
      return;
    }

    port.drain((err) => err ? reject(err) : resolve());
  });
}

async function writeAndDrain(port, chunk) {
  await writeToPort(port, chunk);
  await drainPort(port);
}

/**
 * Reads a file from a serial device using the READFILE command.
 * Buffers until END marker, handles timeouts and errors.
 * @param {SerialPort} port - An open SerialPort instance
 * @param {string} filename - The filename to read (e.g. 'boot.py')
 * @param {number} [timeoutMs] - Optional timeout in ms
 * @returns {Promise<string>} - Resolves with file content, rejects on error/timeout
 */
function readFile(port, filename, timeoutMs = DEFAULT_TIMEOUT) {
  return new Promise(async (resolve, reject) => {
    // Remove all lingering 'data' listeners before starting a new read
    port.removeAllListeners('data');
    // Optionally flush the buffer if available
    if (typeof port.flush === 'function') {
      try {
        await new Promise((res, rej) => {
          port.flush(err => err ? rej(err) : res());
        });
        console.log('[serialFileIO] Serial buffer flushed before readFile:', filename);
      } catch (flushErr) {
        console.warn('[serialFileIO] Serial buffer flush failed before readFile', filename, flushErr);
      }
    }
    
    // Add small delay to let firmware reset completely
    await new Promise(res => setTimeout(res, 50));
    
    let buffer = '';
    let finished = false;
    let resolved = false;
    console.log(`[serialFileIO] readFile called for filename: ${filename}`);
    
    // Add special debugging for boot.py
    const isBootPy = filename === 'boot.py';
    if (isBootPy) {
      console.log(`[serialFileIO][BOOT.PY] Starting boot.py read with ${timeoutMs}ms timeout`);
    }
    
    const timer = setTimeout(() => {
      if (!finished && !resolved) {
        finished = true;
        resolved = true;
        port.off('data', onData);
        console.warn(`[serialFileIO] [TIMEOUT] Listener removed for ${filename}`);
        console.warn(`[serialFileIO] [TIMEOUT] Buffer dump for ${filename}:`, buffer);
        if (isBootPy) {
          console.error(`[serialFileIO][BOOT.PY] TIMEOUT - Buffer length: ${buffer.length} chars`);
          console.error(`[serialFileIO][BOOT.PY] Buffer ends with:`, buffer.slice(-500));
        }
        reject(new Error(`Timeout reading file: ${filename}`));
      }
    }, timeoutMs);
    function cleanupListeners() {
      port.off('data', onData);
      console.log(`[serialFileIO] Listener cleanup for ${filename}`);
    }
    function onData(data) {
      if (resolved) return;
      const str = data.toString();
      buffer += str;
      
      // Enhanced debugging for boot.py
      if (isBootPy) {
        console.log(`[serialFileIO][BOOT.PY] Received ${str.length} chars, total buffer: ${buffer.length}`);
        if (buffer.length % 5000 === 0) { // Log every 5KB
          console.log(`[serialFileIO][BOOT.PY] Progress: ${buffer.length} characters received`);
        }
      }
      
      // Log the first 100 chars of the buffer for every chunk
      const preview = buffer.length > 100 ? buffer.slice(0, 100) + '...' : buffer;
      console.log(`[serialFileIO][DEBUG] Buffer preview for ${filename}:`, preview);
      console.log(`[serialFileIO] Data received for ${filename}:`, str);
      
      // Robust END marker detection: match END_filename on its own line, at end, or with trailing whitespace
      const endMarkerRegex = new RegExp(`END_${filename.replace('.', '\\.')}\\s*$`, 'm');
      if (endMarkerRegex.test(buffer)) {
        if (isBootPy) {
          console.log(`[serialFileIO][BOOT.PY] END marker detected! Buffer length: ${buffer.length}`);
        }
        finished = true;
        resolved = true;
        clearTimeout(timer);
        cleanupListeners();
        // Extract content between START_filename and END_filename markers with enhanced contamination filtering
        const startMarker = `START_${filename}`;
        const endMarker = `END_${filename}`;
        
        let content = '';
        const lines = buffer.split(/\r?\n/);
        let capturing = false;
        let startFound = false;
        
        if (isBootPy) {
          console.log(`[serialFileIO][BOOT.PY] Processing ${lines.length} lines, looking for START_${filename} and END_${filename}`);
        }
        
        for (const line of lines) {
          const trimmed = line.trim();
          
          if (trimmed === startMarker) {
            capturing = true;
            startFound = true;
            content = ''; // Reset content when we find the start marker
            console.log(`[serialFileIO][MARKER] Found START marker for ${filename}`);
            if (isBootPy) {
              console.log(`[serialFileIO][BOOT.PY] START marker found, beginning content capture`);
            }
            continue;
          }
          
          if (trimmed === endMarker) {
            if (startFound) {
              console.log(`[serialFileIO][MARKER] Found END marker for ${filename}`);
              if (isBootPy) {
                console.log(`[serialFileIO][BOOT.PY] END marker found, content length: ${content.length}`);
              }
              capturing = false;
              break;
            } else {
              console.warn(`[serialFileIO][MARKER] Found END marker for ${filename} without START - ignoring`);
              continue;
            }
          }
          
          if (capturing && startFound) {
            // Enhanced contamination filtering - reject lines that look like firmware artifacts
            // Be more specific to avoid filtering legitimate file content
            if (trimmed === 'FIRMWARE_READY:OK' || 
                trimmed.startsWith('FIRMWARE_VERSIONS:') ||
                (trimmed.includes('"code.py"') && trimmed.includes('FIRMWARE_FILES')) ||
                (trimmed.includes('"hardware.py"') && trimmed.includes('FIRMWARE_FILES')) ||
                (trimmed.includes('"utils.py"') && trimmed.includes('FIRMWARE_FILES')) ||
                (trimmed.includes('"gamepad.py"') && trimmed.includes('FIRMWARE_FILES')) ||
                (trimmed.includes('"serial_handler.py"') && trimmed.includes('FIRMWARE_FILES')) ||
                (trimmed.includes('"pin_detect.py"') && trimmed.includes('FIRMWARE_FILES'))) {
              console.log(`[serialFileIO][FILTER] Filtered contamination for ${filename}: ${trimmed}`);
              continue;
            }
            
            if (content.length > 0) content += '\n';
            content += line;
          }
        }
        
        // Additional validation - check if we actually found both markers
        if (!startFound) {
          console.error(`[serialFileIO][ERROR] No START marker found for ${filename} in buffer`);
          if (isBootPy) {
            console.error(`[serialFileIO][BOOT.PY] Buffer first 1000 chars:`, buffer.substring(0, 1000));
            console.error(`[serialFileIO][BOOT.PY] Buffer last 1000 chars:`, buffer.substring(buffer.length - 1000));
          }
          reject(new Error(`No START marker found for ${filename}`));
          return;
        }
        
        content = content.trim();
        
        if (isBootPy) {
          console.log(`[serialFileIO][BOOT.PY] Raw content length before cleanup: ${content.length}`);
        }
        
        // CRITICAL: Remove any trailing END marker that might have been included
        const endMarkerPattern = new RegExp(`\\s*END_${filename.replace('.', '\\.')}\\s*$`);
        if (endMarkerPattern.test(content)) {
          content = content.replace(endMarkerPattern, '').trim();
          console.log(`[serialFileIO][CLEANUP] Removed trailing END marker from ${filename} content`);
          if (isBootPy) {
            console.log(`[serialFileIO][BOOT.PY] Content length after END marker cleanup: ${content.length}`);
          }
        }
        
        if (isBootPy) {
          console.log(`[serialFileIO][BOOT.PY] Final content validation:`);
          console.log(`[serialFileIO][BOOT.PY] - Length: ${content.length} characters`);
          console.log(`[serialFileIO][BOOT.PY] - Contains __version__: ${content.includes('__version__')}`);
          console.log(`[serialFileIO][BOOT.PY] - Contains final print: ${content.includes('print(f"BGG Guitar Controller v{__version__} boot complete!")')}`);
          console.log(`[serialFileIO][BOOT.PY] - Starts with: ${content.substring(0, 100)}`);
          console.log(`[serialFileIO][BOOT.PY] - Ends with: ${content.substring(content.length - 100)}`);
        }
        
        // Log the first 100 chars of the final content
        const contentPreview = content.length > 100 ? content.slice(0, 100) + '...' : content;
        console.log(`[serialFileIO][DEBUG] Final content preview for ${filename}:`, contentPreview);
        console.log(`[serialFileIO] Final content for ${filename}:`, content);
        console.log(`[serialFileIO] Promise resolved for ${filename}`);
        resolve(content);
      }
    }
    port.on('data', onData);
    port.write(`READFILE:${filename}\n`);
  });
}

/**
 * Writes a file to a serial device using the WRITEFILE command.
 * Handles timeouts and errors.
 * @param {SerialPort} port - An open SerialPort instance
 * @param {string} filename - The filename to write (e.g. 'user_presets.json')
 * @param {string|Buffer} content - The file content to write
 * @param {number} [timeoutMs] - Optional timeout in ms
 * @returns {Promise<boolean>} - Resolves true on success, rejects on error/timeout
 */
function writeFile(port, filename, content, timeoutMs = null) {
  return new Promise(async (resolve, reject) => {
    let payload = content;
    if (typeof payload === 'string' && filename.toLowerCase().endsWith('.json') && !payload.includes('\n') && payload.length > 256) {
      try {
        payload = JSON.stringify(JSON.parse(payload), null, 2);
        console.log(`[serialFileIO] Expanded minified JSON payload for ${filename} into multi-line transport form`);
      } catch (jsonExpandError) {
        console.warn(`[serialFileIO] Failed to expand JSON payload for ${filename}, sending original content`, jsonExpandError);
      }
    }

    let ackReceived = false;
    let errorReceived = false;
    let responseBuffer = '';
    let readyReceived = false;
    const allResponses = []; // Track all responses for debugging
    
    // Calculate dynamic timeout based on file size if not specified
    const contentLength = typeof payload === 'string' ? payload.length : payload.byteLength || payload.length;
    const fileSizeKB = Math.ceil(contentLength / 1024);
    
    // Enhanced timeout calculation: base + per-KB + large file bonus
    let calculatedTimeout = DEFAULT_TIMEOUT + (fileSizeKB * TIMEOUT_PER_KB);
    if (fileSizeKB > 10) {
      calculatedTimeout += LARGE_FILE_BONUS; // Extra time for files > 10KB
    }
    
    const finalTimeout = timeoutMs || calculatedTimeout;
    
    console.log(`[serialFileIO] writeFile starting for ${filename}`);
    console.log(`[serialFileIO] File size: ${contentLength} bytes (${fileSizeKB} KB), timeout: ${finalTimeout}ms`);
    
    // Special handling for CircuitPython system files that trigger automatic reboot
    // Only applies when writing to root directory, not to updates folder
    const isSystemFile = (filename === 'boot.py' || filename === 'code.py') && !filename.includes('/');
    
    if (isSystemFile) {
      console.log(`[serialFileIO] Writing system file ${filename} - CircuitPython will reboot immediately`);
    }
    
    const actualTimeout = isSystemFile ? 45000 : finalTimeout; // 45 second timeout for system files to handle large files and reboot time
    
    const timer = setTimeout(() => {
      if (!ackReceived && !errorReceived) {
        port.off('data', onData);
        port.off('error', onError);
        console.error(`[serialFileIO] TIMEOUT writing ${filename} after ${actualTimeout}ms`);
        console.error(`[serialFileIO] All responses received during timeout:`, allResponses);
        reject(new Error(`Timeout writing file: ${filename}. Responses: ${allResponses.join(', ')}`));
      }
    }, actualTimeout);
    
    let resolveReadyWait = null;
    const readyWaitPromise = new Promise((resolve) => {
      resolveReadyWait = resolve;
    });

    function onData(data) {
      const str = data.toString();
      responseBuffer += str;
      allResponses.push(str.trim()); // Track all responses
      console.log(`[serialFileIO] writeFile received data for ${filename}:`, JSON.stringify(str));

      const readyPattern = new RegExp(`(?:WRITEFILE|STREAM):READY:${filename.replace('.', '\\.')}`, 'i');
      if (!readyReceived && readyPattern.test(responseBuffer)) {
        readyReceived = true;
        console.log(`[serialFileIO] writeFile READY received for ${filename}`);
        if (resolveReadyWait) {
          resolveReadyWait();
          resolveReadyWait = null;
        }
      }
      
      // DEBUG: Check for any device response
      if (str.includes('DEBUG:') || str.includes('Note:') || str.includes('Starting write')) {
        console.log(`[serialFileIO] DEBUG: Device acknowledged command:`, str.trim());
      }
      
      // Completion ACK can be split across serial chunks, so inspect full buffered response.
      if (/File\s+[^\n\r]*written/i.test(responseBuffer)) {
        ackReceived = true;
        clearTimeout(timer);
        port.off('data', onData);
        port.off('error', onError);
        console.log(`[serialFileIO] writeFile SUCCESS for ${filename}`);
        resolve(true);
      } else if ((responseBuffer.includes('ERROR:') || responseBuffer.includes('Error')) &&
                 !responseBuffer.includes('DEBUG: Line received:') &&
                 !responseBuffer.includes('DEBUG:') &&
                 !responseBuffer.trim().startsWith('print(') &&
                 !responseBuffer.includes('# ') &&
                 !responseBuffer.includes('"""') &&
                 !responseBuffer.includes("'''")) {
        // Only treat as real error if not part of echoed code content
        errorReceived = true;
        clearTimeout(timer);
        port.off('data', onData);
        port.off('error', onError);
        console.error(`[serialFileIO] writeFile ERROR for ${filename}. All responses:`, allResponses);
        reject(new Error(`Error writing file: ${filename}. Error: ${responseBuffer.trim()}`));
      } else if (str.includes('FIRMWARE_READY')) {
        console.log(`[serialFileIO] Received FIRMWARE_READY during ${filename} write - ignoring`);
      } else {
        console.log(`[serialFileIO] Received unexpected response during ${filename} write:`, JSON.stringify(str));
      }
    }
    
    // Add error handler for port errors
    function onError(error) {
      console.error(`[serialFileIO] Port error during ${filename} write:`, error);
      clearTimeout(timer);
      port.off('data', onData);
      port.off('error', onError);
      reject(error);
    }
    
    port.on('data', onData);
    port.on('error', onError);
    console.log(`[serialFileIO] Sending WRITEFILE command for ${filename}`);
    await writeAndDrain(port, `WRITEFILE:${filename}\n`);

    // Wait for the device to acknowledge that it has entered write mode before
    // streaming file content. A blind delay is not sufficient on slower polls.
    await Promise.race([
      readyWaitPromise,
      new Promise((_, rejectReady) => setTimeout(() => rejectReady(new Error(`Timeout waiting for write readiness: ${filename}`)), 2000))
    ]);
    
    console.log(`[serialFileIO] Sending content for ${filename} (${contentLength} bytes)`);
    const isJsonFile = filename.toLowerCase().endsWith('.json');
    
    // For very large files (>30KB), use chunk-based transmission to prevent memory allocation failures
    if (fileSizeKB > 30) {
      console.log(`[serialFileIO] Using chunk-based transmission for large file ${filename} (${fileSizeKB}KB)`);
      
      // Send content in smaller chunks to prevent device memory allocation failures
      const chunkSize = 1024; // 1KB chunks to prevent memory issues
      const chunks = [];
      
      for (let i = 0; i < payload.length; i += chunkSize) {
        chunks.push(payload.slice(i, i + chunkSize));
      }
      
      console.log(`[serialFileIO] Sending ${chunks.length} chunks of ~${chunkSize} bytes each for ${filename}`);
      
      for (let chunkIndex = 0; chunkIndex < chunks.length; chunkIndex++) {
        const chunk = chunks[chunkIndex];
        await writeAndDrain(port, chunk);
        
        // Add delay between chunks to allow device to process and write to storage
        await new Promise(resolve => setTimeout(resolve, 200)); // 200ms between chunks
        
        // Log progress every 10 chunks
        if ((chunkIndex + 1) % 10 === 0 || chunkIndex === chunks.length - 1) {
          console.log(`[serialFileIO] Sent chunk ${chunkIndex + 1}/${chunks.length} for ${filename}`);
        }
      }
    } else {
      // Send content line by line with delays to prevent overwhelming the device
      const lines = String(payload).split('\n');
      console.log(`[serialFileIO] Sending ${lines.length} lines for ${filename}`);
      
      for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        // Send line with newline (except for last line if content didn't end with newline)
        if (i < lines.length - 1 || String(payload).endsWith('\n')) {
          await writeAndDrain(port, line + '\n');
        } else {
          await writeAndDrain(port, line);
        }

        if (isJsonFile) {
          await new Promise(resolve => setTimeout(resolve, 12));
        }
        
        // Add delay every 10 lines to give device time to process
        if ((i + 1) % 10 === 0) {
          // Enhanced delays: 75ms for large files (>10KB), 50ms for smaller files
          const delayMs = isJsonFile ? 120 : (fileSizeKB > 10 ? 75 : 50);
          await new Promise(resolve => setTimeout(resolve, delayMs));
          // Log progress for large files (50+ lines)
          if (lines.length >= 50) {
            console.log(`[serialFileIO] Sent ${i + 1}/${lines.length} lines for ${filename}`);
          }
        }
      }
    }
    
    // Ensure proper line ending before END command
    if (!String(payload).endsWith('\n')) {
      await writeAndDrain(port, '\n');
    }
    
    // Add a delay before sending END command
    await new Promise(resolve => setTimeout(resolve, isJsonFile ? 750 : 100));
    await writeAndDrain(port, 'END\n');
    console.log(`[serialFileIO] writeFile commands sent for ${filename}`);
  });
}

module.exports = {
  readFile,
  writeFile
};
