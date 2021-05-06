import { logging } from 'stencila'

/**
 * Subscribe to stdout log messages from Stencila CLI client and log to Electron app console.
 */
export const debug = () => {
  logging.toConsole()
}
