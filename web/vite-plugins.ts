import {
  existsSync,
  mkdirSync,
  renameSync,
  unlinkSync,
  writeFileSync,
} from 'fs'
import { dirname, join, resolve } from 'path'
import { fileURLToPath } from 'url'
import type { Plugin } from 'vite'

/**
 * Lit packages use `.js` extensions in their `exports` maps
 * (e.g. `./decorators.js`, `./directives/ref.js`). This plugin
 * lets source files use extensionless imports (e.g. `lit/decorators`)
 * by automatically appending `.js` when Vite can't resolve the bare path.
 */
export function litResolve(): Plugin {
  const litPackages = ['lit', '@lit-labs/observers']
  return {
    name: 'lit-resolve',
    enforce: 'pre',
    resolveId(source) {
      for (const pkg of litPackages) {
        if (source.startsWith(`${pkg}/`) && !source.endsWith('.js')) {
          return this.resolve(`${source}.js`)
        }
      }
      return null
    },
  }
}

/**
 * Fix CSS output paths and clean up theme stub JS files.
 *
 * Rollup extracts CSS from entry points as "assets" which lose their
 * entry path prefix. This plugin moves them to match the entry name
 * (e.g., `base.css` → `themes/base.css`, `dynamic.css` → `views/dynamic.css`).
 *
 * It also removes the empty JS stubs generated from CSS-only theme entries.
 */
export function fixCssPaths(outDir: string): Plugin {
  const distDir = resolve(dirname(fileURLToPath(import.meta.url)), outDir)

  // Map CSS basename → target directory based on entry point names
  const cssPathMap: Record<string, string> = {
    'base.css': 'themes',
    'stencila.css': 'themes',
    'latex.css': 'themes',
    'tufte.css': 'themes',
    'dynamic.css': 'views',
    'vscode.css': 'views',
  }

  return {
    name: 'fix-css-paths',
    writeBundle(_, bundle) {
      // Move CSS files to their correct directories
      for (const [cssName, targetDir] of Object.entries(cssPathMap)) {
        const src = join(distDir, cssName)
        if (existsSync(src)) {
          const dest = join(distDir, targetDir, cssName)
          mkdirSync(dirname(dest), { recursive: true })
          renameSync(src, dest)
          for (const ext of ['.br', '.gz', '.map']) {
            const srcExt = join(distDir, `${cssName}${ext}`)
            if (existsSync(srcExt)) {
              renameSync(srcExt, join(distDir, targetDir, `${cssName}${ext}`))
            }
          }
        }
      }

      // Remove empty JS stubs from CSS-only theme entries
      for (const fileName of Object.keys(bundle)) {
        if (
          fileName.startsWith('themes/') &&
          fileName.endsWith('.js') &&
          fileName !== 'themes/init.js'
        ) {
          for (const ext of ['', '.map', '.br', '.gz']) {
            try {
              unlinkSync(join(distDir, `${fileName}${ext}`))
            } catch {
              // ignore
            }
          }
        }
      }

      // Restore .gitignore so the dist/ directory is tracked by git
      // (Vite's emptyOutDir: true deletes it)
      const gitignore = join(distDir, '.gitignore')
      if (!existsSync(gitignore)) {
        writeFileSync(
          gitignore,
          '# Ignore everything except this file so the directory exists in the repo\n*\n!.gitignore\n'
        )
      }
    },
  }
}
