import * as logga from '@stencila/logga'

/**
 * Configure log event handling
 */
export const configure = (debug: boolean = false): void => {
  logga.replaceHandlers((data: logga.LogData): void => {
    logga.defaultHandler(data, {
      level: debug ? logga.LogLevel.debug : logga.LogLevel.info,
      throttle: {
        // Do not repeat the same message within 5s
        signature: `${data.tag}${data.level}${data.message}`,
        duration: 5000
      }
    })
  })
}
