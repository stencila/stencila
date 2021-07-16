import { config, dispatch, plugins, Result } from 'stencila'
import { CHANNEL } from '../../preload/channels'
import {
  ConfigWindowOpen,
  NormalizedPlugins,
  PluginsInstall,
  PluginsList,
  PluginsRefresh,
  PluginsUninstall,
  PluginsUpgrade,
  ReadConfig,
} from '../../preload/types'
import { makeHandlers, removeChannelHandlers } from '../utils/handler'
import { handle, valueToSuccessResult } from '../utils/ipc'
import { CONFIG_CHANNEL } from './channels'
import { showSettings } from './window'

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

const registerConfigHandlers = () => {
  handle<ConfigWindowOpen>(CHANNEL.CONFIG_WINDOW_OPEN, async () => {
    showSettings()
    return valueToSuccessResult()
  })

  handle<ReadConfig>(CHANNEL.CONFIG_READ, async () => getConfig())

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
