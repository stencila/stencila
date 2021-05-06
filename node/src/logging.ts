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

import { subscribe, unsubscribe } from "./pubsub"

const addon = require('../index.node')

// Initialize this module (sets up logging handlers on the Rust side)
addon.loggingInit()

/**
 * Send pubsub events on the "logging" topic to Node's `console` object.
 *  
 * Call `toConsole(false)` to turn this off (e.g. if you want to send
 * to desktop notifications)
 */
export function toConsole(on = true): void {
  if (on) {
    subscribe("logging", (_topic: string, event: any) => {
      let message = event.message
      switch (event.metadata.level) {
        case 'TRACE':
          return console.trace(message)
        case 'DEBUG':
          return console.debug(message)
        case 'INFO':
          return console.info(message)
        case 'WARN':
          return console.warn(message)
        case 'ERROR':
          return console.error(message)
      }
    })
  } else {
    unsubscribe("logging")
  }
}

// By default log to console
toConsole(true)

/**
 * Test that Rust to Node.js logging is working by emitting some log events from Rust.
 */
export function test(): void {
  return addon.loggingTest()
}
