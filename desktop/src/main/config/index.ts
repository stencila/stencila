import { ipcMain } from 'electron'
import { config, dispatch, plugins, Result } from 'stencila'
import { CHANNEL } from '../../preload/channels'
import { NormalizedPlugins } from '../../preload/types'
import { removeChannelHandlers } from '../utils/handler'
import { valueToSuccessResult } from '../utils/rpc'
import { CONFIG_CHANNEL } from './channels'

const getConfig = async () => {
  return valueToSuccessResult({
    config: config.get(),
    schema: config.schema(),
  })
}

const getPlugins = (): Result<NormalizedPlugins> => {
  return valueToSuccessResult(
    plugins.list().reduce(
      (pluginObject: NormalizedPlugins, plugin) => {
        return {
          entities: { ...pluginObject.entities, [plugin.name]: plugin },
          ids: [...pluginObject.ids, plugin.name],
        }
      },
      { entities: {}, ids: [] }
    )
  )
}

export const registerConfigHandlers = () => {
  try {
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
      return dispatch.plugins.install(name)
    })

    ipcMain.handle(CHANNEL.UNINSTALL_PLUGIN, async (_event, name) => {
      return dispatch.plugins.uninstall(name)
    })

    ipcMain.handle(CHANNEL.UPGRADE_PLUGIN, async (_event, name) => {
      return dispatch.plugins.upgrade(name)
    })

    ipcMain.handle(
      CHANNEL.REFRESH_PLUGINS,
      async (_event, pluginList: string[]) => {
        return dispatch.plugins.refresh(
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
