import { existsSync, unlinkSync, writeFileSync } from 'fs'
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
 * Clean up build artifacts after Vite writes the bundle.
 *
 * Removes empty JS stubs generated for CSS-only theme entries and restores the
 * output `.gitignore` after `emptyOutDir` clears the directory.
 */
export function finalizeBuildArtifacts(outDir: string): Plugin {
  const distDir = resolve(dirname(fileURLToPath(import.meta.url)), outDir)
  const themeStubEntries = new Set([
    'themes/base.js',
    'themes/latex.js',
    'themes/tufte.js',
  ])

  return {
    name: 'finalize-build-artifacts',
    writeBundle(_, bundle) {
      // Remove empty JS stubs from CSS-only theme entries
      for (const fileName of Object.keys(bundle)) {
        if (themeStubEntries.has(fileName)) {
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
