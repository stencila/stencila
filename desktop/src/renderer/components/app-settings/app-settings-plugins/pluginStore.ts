import { createStore } from '@stencil/store'
import { Plugin } from 'stencila'
import { client } from '../../../client'

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
    await client.plugins.refresh(pluginList)
  }

  const plugins = await client.plugins.list().then(({ value }) => {
    pluginStore.plugins = value
    return value
  })

  return plugins
}
