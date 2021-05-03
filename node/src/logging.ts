// Node.js bindings for ../../rust/src/logging.rs, see there for more documentation.

import { Config } from './config'
import { toJSON } from './prelude'

const addon = require('../index.node')

export function init(config?: Config): void {
  return addon.loggingInit(config ? toJSON(config) : '')
}

export function test(): void {
  return addon.loggingTest()
}
