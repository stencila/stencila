import { globalHandlers } from './global'
import { launcherHandlers } from './launcher'
import { registerMenu } from './menu'
import { appStoreHandlers } from './store'
import { setErrorReportingId } from './utils/errors'
import { checkForUpdates } from './utils/update'

export const main = () => {
  setErrorReportingId()
  checkForUpdates()
  appStoreHandlers.register(null)
  globalHandlers.register(null)
  launcherHandlers.register(null)
  registerMenu()
}
