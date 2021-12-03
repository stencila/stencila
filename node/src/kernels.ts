import { fromJSON } from './prelude'

const addon = require('../index.node')

/**
 * Get the list of available kernels
 */
export function available(): string[] {
  return fromJSON<string[]>(addon.kernelsAvailable())
}
