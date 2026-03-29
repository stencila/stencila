import { dirname, resolve } from 'path'
import { fileURLToPath } from 'url'
import { defineConfig } from 'vite'
import { finalizeBuildArtifacts, litResolve } from './vite-plugins'

const __dirname = dirname(fileURLToPath(import.meta.url))

export default defineConfig({
  build: {
    outDir: '../vscode/out/web',
    emptyOutDir: true,
    sourcemap: false,
    target: 'es2020',
    rollupOptions: {
      input: {
        'views/vscode': resolve(__dirname, 'src/views/vscode.ts'),
        'themes/base': resolve(__dirname, 'src/themes/_base.ts'),
        'themes/latex': resolve(__dirname, 'src/themes/_latex.ts'),
        'themes/tufte': resolve(__dirname, 'src/themes/_tufte.ts'),
      },
      output: {
        entryFileNames: '[name].js',
        chunkFileNames: '[name]-[hash].js',
        assetFileNames: (assetInfo) => {
          const originalFileNames = assetInfo.originalFileNames ?? []

          if (originalFileNames.some((fileName) => fileName.endsWith('src/views/vscode.ts'))) {
            return 'views/vscode[extname]'
          }

          if (originalFileNames.some((fileName) => fileName.endsWith('src/themes/_base.ts'))) {
            return 'themes/base[extname]'
          }

          if (originalFileNames.some((fileName) => fileName.endsWith('src/themes/_latex.ts'))) {
            return 'themes/latex[extname]'
          }

          if (originalFileNames.some((fileName) => fileName.endsWith('src/themes/_tufte.ts'))) {
            return 'themes/tufte[extname]'
          }

          return '[name][extname]'
        },
      },
    },
  },
  plugins: [litResolve(), finalizeBuildArtifacts('../vscode/out/web')],
})
