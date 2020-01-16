/**
 * Log configuration for development scripts in this repo.
 *
 * To get `DEBUG` level log entries, set the `DEBUG` env var. e.g
 *
 * ```bash
 * DEBUG=1 npx ts-node ts/bindings/schema.ts
 * ```
 */

import * as logga from '@stencila/logga'

logga.replaceHandlers((data: logga.LogData): void => {
  if (data.level <= (process.env.DEBUG !== undefined ? 3 : 2)) {
    // Don't print noisy stack traces
    data.stack = ''
    logga.defaultHandler(data)
  }
})

const log = logga.getLogger('schema')
export default log
