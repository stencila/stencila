import { config, dispatch, plugins, Result } from 'stencila'
import { CHANNEL } from '../../preload/channels'
import {
  NormalizedPlugins,
  PluginsInstall,
  PluginsList,
  PluginsRead,
  PluginsRefresh,
  PluginsUninstall,
  PluginsUpgrade,
  ReadConfig,
} from '../../preload/types'
import { removeChannelHandlers } from '../utils/handler'
import { handle, valueToSuccessResult } from '../utils/ipc'
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
    handle<ReadConfig>(CHANNEL.CONFIG_READ, async () => getConfig())

    handle<PluginsRead>(CHANNEL.READ_PLUGINS, async () => {
      return getPlugins()
    })

    handle<PluginsList>(CHANNEL.PLUGINS_LIST, async () => {
      plugins.refresh([])
      return getPlugins()
    })

    handle<PluginsInstall>(CHANNEL.PLUGINS_INSTALL, async (_event, name) => {
      return dispatch.plugins.install(name)
    })

    handle<PluginsUninstall>(CHANNEL.PLUGINS_UNINSTALL, async (_event, name) => {
      return dispatch.plugins.uninstall(name)
    })

    handle<PluginsUpgrade>(CHANNEL.PLUGIN_UPGRADE, async (_event, name) => {
      return dispatch.plugins.upgrade(name)
    })

    handle<PluginsRefresh>(
      CHANNEL.PLUGINS_REFRESH,
      async (_event, pluginList) => {
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
