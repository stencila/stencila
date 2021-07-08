import { ipcMain } from 'electron'
import { NormalizedPlugins } from '../../preload/types'
import { config, plugins } from 'stencila'
import { CHANNEL } from '../../preload/channels'
import { removeChannelHandlers } from '../utils/handler'
import { CONFIG_CHANNEL } from './channels'
import { showSettings } from './window'

export const getConfig = async () => {
  return {
    config: config.get(),
    schema: config.schema(),
  }
}

export const getPlugins = (): NormalizedPlugins => {
  return plugins.list().reduce(
    (pluginObject: NormalizedPlugins, plugin) => {
      return {
        entities: { ...pluginObject.entities, [plugin.name]: plugin },
        ids: [...pluginObject.ids, plugin.name],
      }
    },
    { entities: {}, ids: [] }
  )
}

export const registerConfigHandlers = () => {
  try {
    ipcMain.handle(CHANNEL.OPEN_CONFIG_WINDOW, async () => {
      showSettings()
    })

    ipcMain.handle(CHANNEL.READ_CONFIG, async () => {
      return getConfig()
    })

    ipcMain.handle(CHANNEL.READ_PLUGINS, async () => {
      return getPlugins()
    })

    ipcMain.handle(CHANNEL.LIST_AVAILABLE_PLUGINS, async () => {
      plugins.refresh([])
      return getPlugins()
    })

    ipcMain.handle(CHANNEL.INSTALL_PLUGIN, async (_event, name) => {
      return plugins.install(name)
    })

    ipcMain.handle(CHANNEL.UNINSTALL_PLUGIN, async (_event, name) => {
      return plugins.uninstall(name)
    })

    ipcMain.handle(CHANNEL.UPGRADE_PLUGIN, async (_event, name) => {
      return plugins.upgrade(name)
    })

    ipcMain.handle(
      CHANNEL.REFRESH_PLUGINS,
      async (_event, pluginList: string[] = []) => {
        return plugins.refresh(
          pluginList ?? plugins.list().map((plugin) => plugin.name)
        )
      }
    )
  } catch {
    // Handlers likely already registered
  }
}

export const removeConfigHandlers = () => {
  removeChannelHandlers(CONFIG_CHANNEL)
}
