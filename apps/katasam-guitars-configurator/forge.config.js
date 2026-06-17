const { FusesPlugin } = require('@electron-forge/plugin-fuses');
const { FuseV1Options, FuseVersion } = require('@electron/fuses');

module.exports = {
  packagerConfig: {
    asar: true,
    // Forge appends the right extension per platform (.ico on Windows, .icns on macOS)
    icon: './bg-bee-icon',
    name: 'KATASAM Guitars Configurator',
    files:[
      'renderer/**/*',
      'serial/**/*',
      'main.js',
      'preload.js',
      'device.js',
      'package.json'
    ]
  },
  rebuildConfig: {},
  makers: [
    {
      name: '@electron-forge/maker-squirrel',
      config: {
        setupExe: 'KATASAM Guitars Configurator Setup.exe',
        setupIcon: './bg-bee-icon.ico',
        iconUrl: 'https://raw.githubusercontent.com/wattsy74/bgg-windows-app/main/bg-bee-icon.ico'
      },
    },
    {
      name: '@rabbitholesyndrome/electron-forge-maker-portable',
      config: {
        icon: './bg-bee-icon.ico',
        portable: {
          artifactName: 'KATASAM-Guitars-Configurator-v${version}-portable.exe',
          requestExecutionLevel: 'user'
        }
      }
    },
    {
      name: '@electron-forge/maker-dmg',
      platforms: ['darwin'],
      config: {
        name: 'KATASAM Guitars Configurator',
        icon: './bg-bee-icon.icns',
        format: 'ULFO'
      }
    },
    {
      name: '@electron-forge/maker-zip',
      platforms: ['darwin'],
    },
    {
      name: '@electron-forge/maker-deb',
      config: {},
    },
    {
      name: '@electron-forge/maker-rpm',
      config: {},
    },
  ],
  plugins: [
    {
      name: '@electron-forge/plugin-auto-unpack-natives',
      config: {},
    },
    new FusesPlugin({
      version: FuseVersion.V1,
      [FuseV1Options.RunAsNode]: false,
      [FuseV1Options.EnableCookieEncryption]: true,
      [FuseV1Options.EnableNodeOptionsEnvironmentVariable]: false,
      [FuseV1Options.EnableNodeCliInspectArguments]: false,
      [FuseV1Options.EnableEmbeddedAsarIntegrityValidation]: true,
      [FuseV1Options.OnlyLoadAppFromAsar]: true,
    }),
  ],
};
