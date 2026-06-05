import { constants } from 'zlib'
import { dirname, resolve } from 'path'
import { fileURLToPath } from 'url'
import { defineConfig } from 'vite'
import { compression, defineAlgorithm } from 'vite-plugin-compression2'
import { finalizeBuildArtifacts, litResolve } from './vite-plugins'

const __dirname = dirname(fileURLToPath(import.meta.url))
const version = process.env.VERSION
const baseUrl = process.env.BASE_URL

/**
 * Resolve the public base URL for bundled web assets.
 *
 * This affects not only entry script URLs but also Vite-generated preload and
 * lazy-chunk URLs. If left as `/`, some lazily loaded bundles are requested
 * from the site root (e.g. `/dist-*.js`) instead of the Stencila web asset
 * base, causing 404s.
 *
 * Priority order:
 * 1. `BASE_URL` for CDN-published distributions, e.g.
 *    `https://stencila.dev/web/v2.14.1/` or `https://stencila.dev/web/dev/`
 * 2. `VERSION=dev` (or unset) for same-origin server assets at `/~static/dev/`
 * 3. `VERSION=<release>` for same-origin server assets at `/~static/<version>/`
 */
function basePath(): string {
  if (baseUrl) {
    return baseUrl.endsWith('/') ? baseUrl : `${baseUrl}/`
  }

  if (!version || version === 'dev') {
    return '/~static/dev/'
  }

  return `/~static/${version}/`
}

const ASSET_FILE_NAME_RULES = [
  ['src/views/dynamic.ts', 'views/dynamic[extname]'],
  ['src/views/edit.ts', 'views/edit[extname]'],
  ['src/views/graph.ts', 'views/graph[extname]'],
  ['src/themes/_base.ts', 'themes/base[extname]'],
  ['src/themes/_latex.ts', 'themes/latex[extname]'],
  ['src/themes/_tufte.ts', 'themes/tufte[extname]'],
] as const

function assetFileName(assetInfo: { originalFileNames?: string[] }): string {
  const originalFileNames = assetInfo.originalFileNames ?? []
  const rule = ASSET_FILE_NAME_RULES.find(([sourceFile]) =>
    originalFileNames.some((fileName) => fileName.endsWith(sourceFile))
  )

  return rule?.[1] ?? '[name][extname]'
}

export default defineConfig({
  base: basePath(),
  build: {
    outDir: 'dist',
    emptyOutDir: true,
    sourcemap: true,
    target: 'es2020',
    rollupOptions: {
      input: {
        site: resolve(__dirname, 'src/site.ts'),
        'views/dynamic': resolve(__dirname, 'src/views/dynamic.ts'),
        'views/edit': resolve(__dirname, 'src/views/edit.ts'),
        'views/graph': resolve(__dirname, 'src/views/graph.ts'),
        'views/static': resolve(__dirname, 'src/views/static.ts'),
        'themes/init': resolve(__dirname, 'src/themes/init.js'),
        'themes/base': resolve(__dirname, 'src/themes/_base.ts'),
        'themes/latex': resolve(__dirname, 'src/themes/_latex.ts'),
        'themes/tufte': resolve(__dirname, 'src/themes/_tufte.ts'),
      },
      output: {
        entryFileNames: '[name].js',
        chunkFileNames: '[name]-[hash].js',
        assetFileNames: assetFileName,
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
    finalizeBuildArtifacts('dist'),
  ],
})
