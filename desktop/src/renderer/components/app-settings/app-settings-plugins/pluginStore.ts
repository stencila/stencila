import { createStore } from '@stencil/store'
import { plugins } from 'stencila'
import { CHANNEL } from '../../../../preload/index'

interface PluginStore {
  plugins: {
    entities: Record<string, plugins.Plugin>
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

export const getAvailablePlugins = async () => {
  const plugins = (await window.api.invoke(
    CHANNEL.LIST_AVAILABLE_PLUGINS
  )) as any
  pluginStore.plugins = plugins
  return plugins
}
