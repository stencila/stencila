import { ipcMain } from 'electron'
import { config, plugins } from 'stencila'
import { CHANNEL } from '../../preload'
import { showSettings } from './window'

export const getConfig = () => {
  return {
    config: config.read(),
    schema: config.schema(),
  }
}

export const getPlugins = () => plugins.list()

export const registerConfigHandlers = () => {
  ipcMain.handle(CHANNEL.SHOW_CONFIG_WINDOW, async () => {
    return showSettings()
  })

  ipcMain.handle(CHANNEL.READ_CONFIG, async () => {
    return getConfig()
  })

  ipcMain.handle(CHANNEL.READ_PLUGINS, async () => {
    return getPlugins()
  })
}
