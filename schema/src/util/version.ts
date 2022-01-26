import { version as pkgVersion } from '../../package.json'

/**
 * Get the version string (e.g "1.2.3") for this package
 */
export const version = pkgVersion

/**
 * Get the major version string (e.g "1") for this package
 */
export const versionMajor: string = version.split('.')[0]

/**
 * Get the minor version string (e.g "1.2") for this package
 */
export const versionMinor: string = version.split('.').slice(0, 2).join('.')
