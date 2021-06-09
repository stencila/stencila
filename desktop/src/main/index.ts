import { registerGlobalHandlers } from './global'
import { registerMenu } from './menu'
import { registerAppConfigStoreHandlers } from './store'

export const main = () => {
  registerAppConfigStoreHandlers()
  registerGlobalHandlers()
  registerMenu()
}
