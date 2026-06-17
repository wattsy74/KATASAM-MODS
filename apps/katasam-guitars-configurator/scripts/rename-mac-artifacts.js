const fs = require('fs');
const path = require('path');

const root = process.cwd();
const pkg = require(path.join(root, 'package.json'));

const APP_BASENAME = 'KATASAM-Configurator';
const version = pkg.version;
const outMakeDir = path.join(root, 'out', 'make');

function exists(filePath) {
  try {
    fs.accessSync(filePath);
    return true;
  } catch {
    return false;
  }
}

function listFiles(dirPath) {
  if (!exists(dirPath)) return [];
  return fs.readdirSync(dirPath).map((name) => path.join(dirPath, name));
}

function getNewestFile(filePaths) {
  if (filePaths.length === 0) return null;
  return filePaths
    .map((filePath) => ({ filePath, mtimeMs: fs.statSync(filePath).mtimeMs }))
    .sort((a, b) => b.mtimeMs - a.mtimeMs)[0].filePath;
}

function findZipFilesRecursively(dirPath) {
  if (!exists(dirPath)) return [];

  const results = [];
  const entries = fs.readdirSync(dirPath, { withFileTypes: true });

  for (const entry of entries) {
    const fullPath = path.join(dirPath, entry.name);
    if (entry.isDirectory()) {
      results.push(...findZipFilesRecursively(fullPath));
    } else if (entry.isFile() && entry.name.toLowerCase().endsWith('.zip')) {
      results.push(fullPath);
    }
  }

  return results;
}

function safeRename(fromPath, toPath) {
  if (!exists(fromPath)) {
    throw new Error(`Source file does not exist: ${fromPath}`);
  }

  fs.mkdirSync(path.dirname(toPath), { recursive: true });

  if (exists(toPath)) {
    fs.unlinkSync(toPath);
  }

  fs.renameSync(fromPath, toPath);
  console.log(`Renamed: ${path.relative(root, fromPath)} -> ${path.relative(root, toPath)}`);
}

function run() {
  if (!exists(outMakeDir)) {
    console.error('No out/make directory found. Run a mac build first.');
    process.exit(1);
  }

  let renamedAny = false;

  const dmgCandidates = listFiles(outMakeDir)
    .filter((filePath) => filePath.toLowerCase().endsWith('.dmg'))
    .filter((filePath) => fs.statSync(filePath).size > 0);

  const newestDmg = getNewestFile(dmgCandidates);
  if (newestDmg) {
    const dmgTarget = path.join(outMakeDir, `${APP_BASENAME}-v${version}-mac-arm64.dmg`);
    if (path.resolve(newestDmg) !== path.resolve(dmgTarget)) {
      safeRename(newestDmg, dmgTarget);
      renamedAny = true;
    } else {
      console.log(`DMG already standardized: ${path.relative(root, newestDmg)}`);
    }
  } else {
    console.log('No non-empty DMG artifact found to rename.');
  }

  const zipRoot = path.join(outMakeDir, 'zip', 'darwin');
  const zipCandidates = findZipFilesRecursively(zipRoot);
  const newestZip = getNewestFile(zipCandidates);
  if (newestZip) {
    const zipTarget = path.join(
      path.dirname(newestZip),
      `${APP_BASENAME}-v${version}-mac-arm64.zip`
    );
    if (path.resolve(newestZip) !== path.resolve(zipTarget)) {
      safeRename(newestZip, zipTarget);
      renamedAny = true;
    } else {
      console.log(`ZIP already standardized: ${path.relative(root, newestZip)}`);
    }
  } else {
    console.log('No ZIP artifact found to rename.');
  }

  if (!renamedAny) {
    console.log('No artifact rename changes were required.');
  }
}

run();