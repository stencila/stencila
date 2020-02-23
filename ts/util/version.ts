import fs from 'fs'
import path from 'path'

/**
 * Lazily read, cached, package version
 */
let VERSION: string

/**
 * Get the version string (e.g "1.2.3") for this package
 */
export function version(): string {
  if (VERSION === undefined) {
    const json = fs.readFileSync(
      path.join(
        __dirname,
        '..',
        ...(__filename.endsWith('.ts') ? ['..'] : []),
        'package.json'
      ),
      'utf8'
    )
    const pkg = JSON.parse(json)
    VERSION = pkg.version
  }
  return VERSION
}

/**
 * Get the major version string (e.g "1") for this package
 */
export function versionMajor(): string {
  return version().split('.')[0]
}

/**
 * Get the minor version string (e.g "1.2") for this package
 */
export function versionMinor(): string {
  return version()
    .split('.')
    .slice(0, 2)
    .join('.')
}
