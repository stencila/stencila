import { WebContents } from 'electron'
import log, { LogMessage } from 'electron-log'
import { pubsub } from 'stencila'
import { CHANNEL } from './channels'
import { isDevelopment } from './utils/env'

// In memory store of logs. Reset upon application launch
export const logStore: LogMessage[] = []

// Only add log messages `warn` or higher to log file on disk
log.transports.file.level = 'warn'

// Only log messages `warn` or higher to console/log panel when in production
log.transports.console.level = isDevelopment ? 'silly' : 'warn'

// When a logging a message, store it for future retrieval by the "Application Log" window
log.hooks.push((message: LogMessage, transport): LogMessage => {
  if (transport === log.transports.console) {
    // Constrain overall length of the array by dropping the oldest log item
    if (logStore.length > 200) {
      logStore.shift()
    }

    logStore.push(message)
  }
  return message
})

/**
 * Subscribe to stdout log messages from Stencila CLI client and log to Electron app console.
 */
export const enableLogging = () => {
  pubsub.subscribe('logging', (_topic: string, event: any) => {
    const {
      message,
      metadata: { level, target },
    } = event

    const line = [`%c${target} |%c ${message}`, 'color: blue', 'color: unset']

    switch (level) {
      case 'TRACE':
        return log.verbose(...line)
      case 'DEBUG':
        return log.debug(...line)
      case 'INFO':
        return log.info(...line)
      case 'WARN':
        return log.warn(...line)
      case 'ERROR':
        return log.error(...line)
    }
  })
}

export const streamLogsToWindow = (wc: WebContents): void => {
  log.hooks.push((message: LogMessage, transport): LogMessage => {
    if (transport === log.transports.console) {
      wc.send(CHANNEL.LOGS_PRINT, message)
    }
    return message
  })
}
