import replace from '@rollup/plugin-replace'
import { Config } from '@stencil/core'
import { postcss } from '@stencil/postcss'
import dotenv from 'rollup-plugin-dotenv'
import os from 'os'
import { sep } from 'path'

// https://stenciljs.com/docs/config

export const config: Config = {
  srcDir: 'src/renderer',
  globalStyle: 'src/renderer/global/app.css',
  globalScript: 'src/renderer/global/app.ts',
  taskQueue: 'async',
  tsconfig: 'tsconfig.ui.json',
  devServer: {
    openBrowser: false,
  },
  outputTargets: [
    {
      type: 'www',
      // comment the following line to disable service workers in production
      serviceWorker: null,
    },
  ],
  rollupPlugins: {
    before: [
      dotenv(),
      replace({
        'process.env.NODE_ENV': JSON.stringify(process.env.NODE_ENV),
        'process.env.SENTRY_DSN': JSON.stringify(process.env.SENTRY_DSN),
        'process.type': JSON.stringify('renderer'),
        'process.env.OS': JSON.stringify(os.type()),
        'process.env.OS_PATH_SEPARATOR': JSON.stringify(sep),
      }),
    ],
  },
  plugins: [
    postcss({
      plugins: [
        require('tailwindcss')({
          darkMode: 'media',
          purge: ['./src/main/**/*.html', './src/main/**/*.tsx'],
        }),
        require('postcss-nested'),
      ],
    }),
  ],
  testing: {
    transform: {
      '^.+\\.(ts|tsx|jsx|js)$':
        './node_modules/@stencil/core/testing/jest-preprocessor.js',
    },
    testRegex: '/src/.*\\.spec\\.(ts|tsx|js)$',
  },
}
