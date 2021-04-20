// Node.js bindings for ../../rust/src/logging.rs, see there for more documentation.

const addon = require('../index.node')

export function init(): Plugin[] {
  return addon.loggingInit()
}

export function test(): Plugin[] {
  return addon.loggingTest()
}
