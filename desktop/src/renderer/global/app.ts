import { enableCrashReports } from '../../preload/errors'
import { isProduction } from '../../preload/utils/env'
import { isErrorReportingEnabled } from '../store/appConfig'
import { showUnhandledErrors } from '../utils/errors'

/**
 * The code to be executed should be placed within a default function that is
 * exported by the global script. Ensure all of the * code in the global script
 * is wrapped in the function that is exported.
 * @see https://stenciljs.com/docs/config#globalscript
 */
export default async () => {
  // Due to `nodeIntegration: false` and `contextIsolation: true`, Sentry needs
  // to be instantiated in both the `preload` script AND here, the `web` context.
  if (process.env.SENTRY_DSN && isProduction) {
    enableCrashReports(isErrorReportingEnabled)
  }
  showUnhandledErrors()
}
