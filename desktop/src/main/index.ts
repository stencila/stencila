import { registerConfigHandlers } from './config'
import { registerDocumentHandlers } from './document'
import { registerMenu } from './menu'
import { registerProjectHandlers } from './project'

export const main = () => {
  registerMenu()
  registerConfigHandlers()
  registerProjectHandlers()
  registerDocumentHandlers()
}
