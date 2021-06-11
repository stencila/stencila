import { init } from '@sentry/electron'
import { LogEvent, LogLevel } from '@stencila/logga'

export const enableCrashReports = () => {
  if (process.env.SENTRY_DSN && process.env.NODE_ENV === 'production') {
    init({
      dsn: process.env.SENTRY_DSN,
      autoSessionTracking: false
    })
  }
}

export const disableCrashReports = () => {
  // TODO: Disable Sentry
}

export interface LogHandler extends LogEvent {
  level: LogLevel
  error?: Error
}

export const captureError = (error: LogHandler) => {
  // if (getAppConfig(UnprotectedStoreKeys.REPORT_ERRORS)) {
  // TODO: Send errors to Sentry
  console.log(error)
  // }
}
