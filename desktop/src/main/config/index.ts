import { dispatch, plugins, Result } from 'stencila'
import { CHANNEL } from '../../preload/channels'
import {
  ConfigGetAll,
  ConfigSet,
  ConfigWindowOpen,
  NormalizedPlugins,
  PluginsInstall,
  PluginsList,
  PluginsRefresh,
  PluginsUninstall,
  PluginsUpgrade,
} from '../../preload/types'
import { getConfig, setConfig } from './handlers'
import { makeHandlers, removeChannelHandlers } from '../utils/handler'
import { handle, valueToSuccessResult } from '../utils/ipc'
import { CONFIG_CHANNEL } from './channels'
import { showSettings } from './window'

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

const registerConfigHandlers = () => {
  handle<ConfigWindowOpen>(CHANNEL.CONFIG_WINDOW_OPEN, async () => {
    showSettings()
    return valueToSuccessResult()
  })

  handle<ConfigGetAll>(CHANNEL.CONFIG_GET_ALL, async () => {
    return valueToSuccessResult(getConfig())
  })

  handle<ConfigSet>(CHANNEL.CONFIG_SET, async (_event, { key, value }) => {
    setConfig(key, value)
    return valueToSuccessResult()
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
}

const removeConfigHandlers = () => {
  removeChannelHandlers(CONFIG_CHANNEL)
}

export const configHandlers = makeHandlers(
  registerConfigHandlers,
  removeConfigHandlers
)
