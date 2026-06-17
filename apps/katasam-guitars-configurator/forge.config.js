const { FusesPlugin } = require('@electron-forge/plugin-fuses');
const { FuseV1Options, FuseVersion } = require('@electron/fuses');

module.exports = {
  packagerConfig: {
    asar: true,
    // Forge appends the right extension per platform (.ico on Windows, .icns on macOS)
    icon: './icon',
    name: 'KATASAM Configurator',
    ignore: [
      /^\/out($|\/)/,
      /^\/dist($|\/)/,
      /^\/build($|\/)/,
      /^\/scripts($|\/)/,
      /^\/\.git($|\/)/,
      /^\/\.vscode($|\/)/,
      /^\/README\.md$/,
      /^\/API_DOCUMENTATION\.md$/,
      /^\/AUTOMATED_RELEASES\.md$/,
      /^\/AUTO_UPDATE_SYSTEM\.md$/,
      /^\/Bug_and_Features\.md$/,
      /^\/ROADMAP\.md$/,
      /^\/TESTING_DOCUMENT\.md$/,
      /^\/release-config\.json$/,
      /^\/release\.ps1$/,
      /^\/auto-update-presets\.ps1$/,
      /^\/update_and_publish_firmware\.ps1$/,
      /^\/UF2MakerScript\.ps1$/,
      /^\/check_firmware_version\.js$/,
      /^\/debug_boot_write_issue\.js$/,
      /^\/console\.logs$/,
      /^\/test_config_device_name\.json$/,
      /^\/\.DS_Store$/
    ],
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
        setupExe: 'KATASAM Configurator Setup.exe',
        setupIcon: './icon.ico',
        iconUrl: 'https://raw.githubusercontent.com/wattsy74/KATASAM-MODS/main/apps/katasam-guitars-configurator/icon.ico'
      },
    },
    {
      name: '@rabbitholesyndrome/electron-forge-maker-portable',
      config: {
        icon: './icon.ico',
        portable: {
          artifactName: 'KATASAM-Configurator-Portable-${version}.exe',
          requestExecutionLevel: 'user'
        }
      }
    },
    {
      name: '@electron-forge/maker-dmg',
      platforms: ['darwin'],
      config: {
        name: 'KATASAM Configurator',
        icon: './icon.icns',
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
