/**
 * Node.js bindings for ../../rust/src/logging.rs, see there for more documentation.
 *
 * The events published on the "logging" topic by Rust are serializations
 * of `tracing_subscriber`'s events. At the time of writing they looked
 * like this
 *
 *  {
 *    message: 'An info event',
 *    metadata: {
 *      fields: [ 'message' ],
 *      file: 'rust/src/logging.rs',
 *      is_event: true,
 *      is_span: false,
 *      level: 'INFO',
 *      line: 313,
 *      module_path: 'stencila::logging',
 *      name: 'event rust/src/logging.rs:313',
 *      target: 'stencila::logging'
 *    }
 *  }
 */

import { subscribe, unsubscribe } from './pubsub'

const addon = require('../index.node')

// Initialize this module (sets up logging handlers on the Rust side)
if (process.env.NODE_ENV === 'development') {
  const json = JSON.stringify({ logging: { desktop: { level: 'debug' } } })
  addon.loggingInit(json)
} else {
  addon.loggingInit()
}

/**
 * Send pubsub events on the "logging" topic to Node's `console` object.
 *
 * Call `toConsole(false)` to turn this off (e.g. if you want to send
 * to desktop notifications)
 */
export function toConsole(on = true): void {
  if (on) {
    subscribe('logging', (_topic: string, event: any) => {
      const {message, metadata: {level, target}} = event
      const line = `${level} ${target} ${message}`
      switch (level) {
        case 'TRACE':
          return console.trace(line)
        case 'DEBUG':
          return console.debug(line)
        case 'INFO':
          return console.info(line)
        case 'WARN':
          return console.warn(line)
        case 'ERROR':
          return console.error(line)
      }
    })
  } else {
    unsubscribe('logging')
  }
}

/**
 * Test that Rust to Node.js logging is working by emitting some log events from Rust.
 */
export function test(): void {
  return addon.loggingTest()
}
