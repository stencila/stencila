import { ipcMain } from 'electron'
import { config, plugins } from 'stencila'
import { CHANNEL } from '../../preload'
import { showSettings } from './window'

export const getConfig = () => {
  return { config: config.read(), plugins: plugins.list() }
}

export const registerConfigHandlers = () => {
  ipcMain.handle(CHANNEL.SHOW_CONFIG_WINDOW, async () => {
    return showSettings()
  })

  ipcMain.handle(CHANNEL.READ_CONFIG, async () => {
    return getConfig()
  })
}
