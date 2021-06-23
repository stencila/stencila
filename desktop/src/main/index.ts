import { registerGlobalHandlers } from './global'
import { registerMenu } from './menu'
import { registerAppConfigStoreHandlers } from './store'
import { setErrorReportingId } from './utils/errors'

export const main = () => {
  setErrorReportingId()
  registerAppConfigStoreHandlers()
  registerGlobalHandlers()
  registerMenu()
}
