import * as Sentry from '@sentry/electron'
import { LogEvent, LogLevel } from '@stencila/logga'
import { version } from '../../package.json'
import { isProduction } from './utils/env'

export const enableCrashReports = (
  getCrashReportingSetting: () => boolean | Promise<boolean>
) => {
  if (process.env.SENTRY_DSN && isProduction) {
    Sentry.init({
      // debug: true,
      dsn: process.env.SENTRY_DSN,
      tracesSampleRate: 1.0,
      release: version,
      beforeSend: async (event) => {
        const reportingEnabled = await getCrashReportingSetting()

        if (!reportingEnabled) {
          return null
        }

        if (event.user?.ip_address) {
          delete event.user.ip_address
        }

        return event
      },
    })
  }
}

export const setUser = (id: string) => {
  Sentry.setUser({ id })
}

export interface LogHandler extends LogEvent {
  level: LogLevel
  error?: Error
}

export const captureError = (error: Error | PromiseRejectionEvent) => {
  Sentry.captureException(error)
}
