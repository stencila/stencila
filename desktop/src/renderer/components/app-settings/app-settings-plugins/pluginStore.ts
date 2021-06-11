import { createStore } from '@stencil/store'
import { Plugin } from 'stencila'
import { CHANNEL } from '../../../../preload/index'

interface PluginStore {
  plugins: {
    entities: Record<string, Plugin>
    ids: string[]
  }
}

const defaultStore: PluginStore = {
  plugins: {
    entities: {},
    ids: [],
  },
}

export const { state: pluginStore, onChange } = createStore(defaultStore)

export const getAvailablePlugins = async (pluginList?: string[]) => {
  if (pluginList) {
    await window.api.invoke(CHANNEL.REFRESH_PLUGINS, pluginList)
  }

  const plugins = (await window.api.invoke(
    CHANNEL.LIST_AVAILABLE_PLUGINS
  )) as any
  pluginStore.plugins = plugins
  return plugins
}
