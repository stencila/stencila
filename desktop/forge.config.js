/**
 * Electron Forge Configuration
 *
 * See the following examples for guidance:
 *
 * - https://github.com/felixrieseberg/windows95/blob/master/forge.config.js
 * - https://github.com/electron/fiddle/blob/master/forge.config.js
 */
module.exports = {
  packagerConfig: {
    name: 'Stencila',
    // The executableName seems to needs to be the same as the various `exe` and `bin`
    // properties in the makers below
    executableName: 'stencila-desktop',
    appCategoryType: 'public.app-category.productivity',
    icon: './src/assets/icon/stencila',
    osxSign: {
      identity: 'Developer ID Application: Stencila Ltd. (K3PWLCZ5R6)',
      'hardened-runtime': true,
      'gatekeeper-assess': false,
      entitlements: 'entitlements.plist',
      'entitlements-inherit': 'entitlements.plist',
      'signature-flags': 'library',
    },
  },
  publishers: [
    {
      name: '@electron-forge/publisher-github',
      config: {
        repository: {
          owner: 'stencila',
          name: 'stencila',
        },
      },
    },
  ],
  makers: [
    {
      // For more settings see
      // https://js.electronforge.io/maker/squirrel/interfaces/makersquirrelconfig
      name: '@electron-forge/maker-squirrel',
      platforms: ['win32'],
      config: {
        // The `exe` setting is necessary to avoid "File not found: 'Stencila.exe'" error
        // because it defaults to "name field in your app's package.json file with an added .exe extension"
        exe: 'stencila-desktop.exe',
        certificateFile: process.env.WINDOWS_CODESIGN_FILE,
        certificatePassword: process.env.WINDOWS_CODESIGN_PASSWORD,
      },
    },
    {
      name: '@electron-forge/maker-zip',
      platforms: ['darwin'],
    },
    {
      // For more settings see
      // https://js.electronforge.io/maker/deb/interfaces/makerdebconfigoptions
      name: '@electron-forge/maker-deb',
      platforms: ['linux'],
      config: {
        productName: 'Stencila',
        name: 'stencila-desktop',
        icon: './src/assets/icon/stencila.png',
        homepage: 'https://stenci.la',
      },
    },
    {
      name: '@electron-forge/maker-rpm',
      platforms: ['linux'],
    },
  ],
  plugins: [
    [
      '@electron-forge/plugin-webpack',
      {
        mainConfig: './webpack.main.config.js',
        renderer: {
          config: './webpack.renderer.config.js',
          entryPoints: [
            {
              html: './src/renderer/index.html',
              js: './src/renderer/renderer.ts',
              name: 'main_window',
              preload: {
                js: './src/preload/index.ts',
              },
            },
          ],
        },
      },
    ],
    [
      '@electron-forge/plugin-electronegativity',
      {
        isSarif: true,
        parserPlugins: [],
      },
    ],
  ],
}
