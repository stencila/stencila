import { enableCrashReports } from '../preload/errors'
import { globalHandlers } from './global'
import { launcherHandlers } from './launcher'
import { registerMenu } from './menu'
import { appStoreHandlers } from './store'
import { isReportErrorsEnabled } from './store/handlers'
import { setErrorReportingId } from './utils/errors'
import { checkForUpdates } from './utils/update'

/**
 * This function is executed at the earliest possible moment in the app lifecycle.
 * It should configure critical elements which are needed prior to the creation of the main window.
 */
export const prepare = () => {
  enableCrashReports(isReportErrorsEnabled)
  setErrorReportingId()
}

/**
 * This function is executed once the app's main window has been instantiated and handles
 * any remaining setup of the application.
 */
export const main = () => {
  checkForUpdates()
  appStoreHandlers.register(null)
  globalHandlers.register(null)
  launcherHandlers.register(null)
  registerMenu()
}
