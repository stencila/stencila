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
import { handle, valueToSuccessResult } from '../utils/rpc'
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
    handle<ReadConfig>(CHANNEL.READ_CONFIG, async () => getConfig())

    handle<PluginsRead>(CHANNEL.READ_PLUGINS, async () => {
      return getPlugins()
    })

    handle<PluginsList>(CHANNEL.LIST_AVAILABLE_PLUGINS, async () => {
      plugins.refresh([])
      return getPlugins()
    })

    handle<PluginsInstall>(CHANNEL.INSTALL_PLUGIN, async (_event, name) => {
      return dispatch.plugins.install(name)
    })

    handle<PluginsUninstall>(CHANNEL.UNINSTALL_PLUGIN, async (_event, name) => {
      return dispatch.plugins.uninstall(name)
    })

    handle<PluginsUpgrade>(CHANNEL.UPGRADE_PLUGIN, async (_event, name) => {
      return dispatch.plugins.upgrade(name)
    })

    handle<PluginsRefresh>(
      CHANNEL.REFRESH_PLUGINS,
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
