# KATASAM Switcher v1.1.0 — Release Notes

## Bug Fixes

### Firmware downloads now work
The firmware download URLs were pointing to a non-existent GitHub release path (`/releases/download/downloads/...`) that returned HTTP 404. All four firmware URLs have been updated to use the stable latest-release redirect (`/releases/latest/download/...`) which resolves correctly.

### Device detection restored (macOS ARM)
Native modules (`node-hid`, `usb`, `@serialport/bindings-cpp`) were compiled for the wrong CPU architecture, causing an `ERR_DLOPEN_FAILED` crash on every HID device poll. Modules have been rebuilt against the installed Electron version. A `postinstall` script has been added to `package.json` so this rebuilds automatically after any future `npm install`.

### BOOTSEL copy no longer fails with "Destination not writable yet"
The writability pre-check inside the copy helper was testing the filesystem root (`/`) instead of the actual BOOTSEL mount directory (`/Volumes/RPI-RP2`). On macOS the root is never user-writable, so the check always failed. It now correctly checks the destination directory.

### No more duplicate flash attempts (Classic → Santroller and Santroller → Classic)
The renderer kept a `pendingFlash` fallback armed during both flash directions. When `detectDevice` polled every 3 seconds and saw BOOTSEL, it would fire a second concurrent `flashToBootsel` call racing with the main-process watcher — causing one attempt to win, the device to reboot, and the other to hit a false "not writable" or mount-timeout error. The fallback is now disabled for both the Classic and Santroller paths since the main-process watcher handles everything end-to-end.

### `waitForBootselMount` no longer returns prematurely
The mount-ready check was returning `true` the moment it found `hardware_version.txt` (a file from a previous flash), without verifying the volume was actually writable for new files. It now always requires the writability check to pass before declaring the mount ready.

### Hardware version parse failures no longer hide the UI
If hardware version detection returned a null or invalid value, label formatting could fail silently and leave the action buttons hidden. Version parsing now strictly accepts only `v1` or `v2` and treats everything else as `unknown` without breaking the UI.

### Guidance message when hardware version is unknown
When the device is detected but hardware version cannot be determined (no `hardware_version.txt`, no cache, no config), the action buttons are now replaced with a clear message directing the user to use KATASAM Configurator to install firmware and detect hardware version first.

### Copy retry count made consistent
The `start-flash-with-version-detection` path was using only 3 copy retry attempts vs 10 in all other flash paths. Increased to 10 for consistency.

## Other Changes
- Window default size increased to 980×860 with minimum 900×820 so the UI is not cramped on launch.
- `@electron/rebuild` added as a devDependency with a `postinstall` hook.
