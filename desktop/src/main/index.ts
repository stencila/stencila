import { registerConfigHandlers } from './config'
import { registerMenu } from './menu'
import { registerProjectHandlers } from './project'

export const main = () => {
  registerMenu()
  registerConfigHandlers()
  registerProjectHandlers()
}
