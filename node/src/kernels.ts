import { fromJSON } from './prelude'

const addon = require('../index.node')

/**
 * Get the list of languages supported by kernels available on this machine
 */
export function languages(): string[] {
  return fromJSON<string[]>(addon.kernelsLanguages())
}
