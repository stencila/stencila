#!/usr/bin/env node
/**
 * Patch vega packages to add main field for Parcel compatibility.
 *
 * Parcel 2 has issues resolving packages that only use the "exports" field
 * without a fallback "main" field. This script adds the main field to
 * vega packages after npm install.
 */

import { existsSync, readFileSync, writeFileSync } from 'fs'
import { dirname, join } from 'path'
import { fileURLToPath } from 'url'

const __dirname = dirname(fileURLToPath(import.meta.url))
const nodeModulesDir = join(__dirname, '../../node_modules')

const patches = [
  { package: 'vega', main: 'build/vega.module.js' },
  { package: 'vega-lite', main: 'build/index.js' },
  { package: 'vega-embed', main: 'build/embed.js' },
  { package: 'vega-themes', main: 'build/index.js' },
  { package: 'vega-tooltip', main: 'build/index.js' },
  { package: 'vega-interpreter', main: 'build/index.js' },
  { package: 'vega-schema-url-parser', main: 'build/index.js' },
  // vega-canvas needs special handling - use browser build to avoid canvas native module
  { package: 'vega-canvas', main: 'build/vega-canvas.browser.js', browser: 'build/vega-canvas.browser.js' },
]

for (const { package: pkg, main, browser } of patches) {
  const pkgJsonPath = join(nodeModulesDir, pkg, 'package.json')

  if (!existsSync(pkgJsonPath)) {
    console.log(`Skipping ${pkg} - not installed`)
    continue
  }

  try {
    const pkgJson = JSON.parse(readFileSync(pkgJsonPath, 'utf8'))
    let modified = false

    if (!pkgJson.main) {
      pkgJson.main = main
      modified = true
      console.log(`Patched ${pkg}: added main="${main}"`)
    } else {
      console.log(`${pkg} already has main field: ${pkgJson.main}`)
    }

    if (browser && !pkgJson.browser) {
      pkgJson.browser = browser
      modified = true
      console.log(`Patched ${pkg}: added browser="${browser}"`)
    }

    if (modified) {
      writeFileSync(pkgJsonPath, JSON.stringify(pkgJson, null, 2) + '\n')
    }
  } catch (err) {
    console.error(`Error patching ${pkg}:`, err.message)
  }
}
