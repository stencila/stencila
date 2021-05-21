import { registerGlobalHandlers } from './global'
import { registerMenu } from './menu'
import { registerAppConfigStoreHandlers } from './store'
import { setErrorReportingId } from './utils/errors'
import { checkForUpdates } from './utils/update'

export const main = () => {
  setErrorReportingId()
  checkForUpdates()
  registerAppConfigStoreHandlers()
  registerGlobalHandlers()
  registerMenu()
}
