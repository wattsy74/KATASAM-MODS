const fs = require('fs');
const path = require('path');

const root = process.cwd();
const pkg = require(path.join(root, 'package.json'));
const version = pkg.version;
const outMakeDir = path.join(root, 'out', 'make');
const distDir = path.join(root, 'dist');

const APP_NAME = 'KATASAM Configurator';

function walk(dirPath) {
  if (!fs.existsSync(dirPath)) return [];

  const entries = fs.readdirSync(dirPath, { withFileTypes: true });
  const files = [];

  for (const entry of entries) {
    const fullPath = path.join(dirPath, entry.name);
    if (entry.isDirectory()) {
      files.push(...walk(fullPath));
      continue;
    }
    if (entry.isFile()) {
      files.push(fullPath);
    }
  }

  return files;
}

function newest(paths) {
  if (paths.length === 0) return null;
  return paths
    .map((filePath) => ({ filePath, mtime: fs.statSync(filePath).mtimeMs }))
    .sort((a, b) => b.mtime - a.mtime)[0].filePath;
}

function copyArtifact(srcPath, targetFileName) {
  fs.mkdirSync(distDir, { recursive: true });
  const targetPath = path.join(distDir, targetFileName);
  fs.copyFileSync(srcPath, targetPath);
  console.log(`Copied ${path.relative(root, srcPath)} -> ${path.relative(root, targetPath)}`);
}

function run() {
  if (!fs.existsSync(outMakeDir)) {
    console.error('No out/make directory found. Run a build first.');
    process.exit(1);
  }

  const allFiles = walk(outMakeDir);

  const macZipCandidates = allFiles.filter((f) => {
    const lower = f.toLowerCase();
    return lower.endsWith('.zip') && lower.includes(`${path.sep}zip${path.sep}darwin${path.sep}`);
  });
  const macDmgCandidates = allFiles.filter((f) => f.toLowerCase().endsWith('.dmg'));

  const setupExeCandidates = allFiles.filter((f) => {
    const lower = path.basename(f).toLowerCase();
    return lower.endsWith('.exe') && lower.includes('setup');
  });

  const portableExeCandidates = allFiles.filter((f) => {
    const lower = path.basename(f).toLowerCase();
    return lower.endsWith('.exe') && lower.includes('portable');
  });

  const macZip = newest(macZipCandidates);
  const macDmg = newest(macDmgCandidates);
  const setupExe = newest(setupExeCandidates);
  const portableExe = newest(portableExeCandidates);

  if (macZip) {
    copyArtifact(macZip, `${APP_NAME}-${version}-arm64-mac.zip`);
  }

  if (macDmg) {
    copyArtifact(macDmg, `${APP_NAME}-${version}-arm64.dmg`);
  }

  if (setupExe) {
    copyArtifact(setupExe, `${APP_NAME} Setup ${version}.exe`);
  }

  if (portableExe) {
    copyArtifact(portableExe, `KATASAM-Configurator-Portable-${version}.exe`);
  }

  if (!macZip && !macDmg && !setupExe && !portableExe) {
    console.log('No known artifacts found to copy into dist/.');
  }
}

run();
