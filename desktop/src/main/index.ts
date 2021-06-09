import { registerConfigHandlers } from './config'
import { registerDocumentHandlers } from './document'
import { registerGlobalHandlers } from './global'
import { registerLauncherHandlers } from './launcher'
import { registerMenu } from './menu'
import { registerOnboardingHandlers } from './onboarding'
import { registerProjectHandlers } from './project'
import { registerAppConfigStoreHandlers } from './store'

export const main = () => {
  registerAppConfigStoreHandlers()
  registerConfigHandlers()
  registerDocumentHandlers()
  registerGlobalHandlers()
  registerLauncherHandlers()
  registerMenu()
  registerOnboardingHandlers()
  registerProjectHandlers()
}
