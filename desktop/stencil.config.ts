import { Config } from '@stencil/core'
import { postcss } from '@stencil/postcss'

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
      baseUrl: 'https://myapp.local/',
    },
  ],
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
}
