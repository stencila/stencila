import { enableCrashReports } from '../preload/errors'
import { enableLogging } from '../preload/logging'
import { configHandlers } from './config/'
import { isReportErrorsEnabled } from './config/handlers'
import { globalHandlers } from './global'
import { launcherHandlers } from './launcher'
import { registerBaseMenu } from './menu'
import { setErrorReportingId } from './utils/errors'
import { checkForUpdates } from './utils/update'

/**
 * This function is executed at the earliest possible moment in the app lifecycle.
 * It should configure critical elements which are needed prior to the creation of the main window.
 */
export const prepare = () => {
  enableLogging()
  enableCrashReports(isReportErrorsEnabled)
  setErrorReportingId()
}

/**
 * This function is executed once the app's main window has been instantiated and handles
 * any remaining setup of the application.
 */
export const main = () => {
  checkForUpdates()
  configHandlers.register(null)
  globalHandlers.register(null)
  launcherHandlers.register(null)
  registerBaseMenu()
}
