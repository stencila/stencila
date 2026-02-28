import { dirname, resolve } from 'path'
import { fileURLToPath } from 'url'
import { defineConfig } from 'vite'
import { fixCssPaths, litResolve } from './vite-plugins'

const __dirname = dirname(fileURLToPath(import.meta.url))

export default defineConfig({
  build: {
    outDir: '../vscode/out/web',
    emptyOutDir: true,
    sourcemap: false,
    target: 'es2015',
    rollupOptions: {
      input: {
        'views/vscode': resolve(__dirname, 'src/views/vscode.ts'),
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
  plugins: [litResolve(), fixCssPaths('../vscode/out/web')],
})
