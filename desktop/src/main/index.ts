import { registerMenu } from './menu'
import { registerFileHandlers } from './selectFiles'

export const main = () => {
  registerMenu()
  registerFileHandlers()
}
