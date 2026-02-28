import { constants } from 'zlib'
import { dirname, resolve } from 'path'
import { fileURLToPath } from 'url'
import { defineConfig } from 'vite'
import { compression, defineAlgorithm } from 'vite-plugin-compression2'
import { fixCssPaths, litResolve } from './vite-plugins'

const __dirname = dirname(fileURLToPath(import.meta.url))

export default defineConfig({
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    sourcemap: true,
    target: 'es2015',
    rollupOptions: {
      input: {
        'views/dynamic': resolve(__dirname, 'src/views/dynamic.ts'),
        'views/static': resolve(__dirname, 'src/views/static.ts'),
        site: resolve(__dirname, 'src/site.ts'),
        'themes/init': resolve(__dirname, 'src/themes/init.js'),
        'themes/base': resolve(__dirname, 'src/themes/_base.ts'),
        'themes/stencila': resolve(__dirname, 'src/themes/_stencila.ts'),
        'themes/latex': resolve(__dirname, 'src/themes/_latex.ts'),
        'themes/tufte': resolve(__dirname, 'src/themes/_tufte.ts'),
      },
      output: {
        entryFileNames: '[name].js',
        chunkFileNames: '[name]-[hash].js',
        assetFileNames: '[name][extname]',
      },
    },
  },
  plugins: [
    litResolve(),
    compression({
      algorithms: [
        defineAlgorithm('brotliCompress', {
          params: { [constants.BROTLI_PARAM_QUALITY]: constants.BROTLI_MAX_QUALITY },
        }),
      ],
    }),
    fixCssPaths('dist'),
  ],
})
