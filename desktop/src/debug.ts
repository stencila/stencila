import { LogEvent, LogLevel } from '@stencila/logga'
import { logging } from 'stencila'
import { getAppConfig, UnprotectedStoreKeys } from './main/store/handlers'

/**
 * Subscribe to stdout log messages from Stencila CLI client and log to Electron app console.
 */
export const debug = () => {
  logging.toConsole()
}

export interface LogHandler extends LogEvent {
  level: LogLevel
  error?: Error
}

export const captureError = (error: LogHandler) => {
  if (getAppConfig(UnprotectedStoreKeys.REPORT_ERRORS)) {
    // TODO: Send errors to Sentry
    console.log(error)
  }
}
