import { enableCrashReports } from '../../preload/errors'
import { UnprotectedStoreKeys } from '../../preload/stores'
import { isProduction } from '../../preload/utils/env'
import { client } from '../client'
import { showUnhandledErrors } from '../utils/errors'

const isErrorReportingEnabled = () =>
  client.config.ui
    .get(UnprotectedStoreKeys.REPORT_ERRORS)
    .then(({ value }) => (typeof value === 'boolean' ? value : false))

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
