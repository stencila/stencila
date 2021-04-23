// Node.js bindings for ../../rust/src/logging.rs, see there for more documentation.

const addon = require('../index.node')

export function init(): void {
  return addon.loggingInit()
}

export function test(): void {
  return addon.loggingTest()
}
