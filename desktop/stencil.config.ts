import { Config } from '@stencil/core'

// https://stenciljs.com/docs/config

export const config: Config = {
  srcDir: 'src/renderer',
  globalStyle: 'src/renderer/global/app.css',
  globalScript: 'src/renderer/global/app.ts',
  taskQueue: 'async',
  outputTargets: [
    {
      type: 'www',
      // comment the following line to disable service workers in production
      serviceWorker: null,
      baseUrl: 'https://myapp.local/',
    },
  ],
}
