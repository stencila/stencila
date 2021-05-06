import $RefParser from '@apidevtools/json-schema-ref-parser'
import { ipcMain } from 'electron'
import { config, plugins } from 'stencila'
import { CHANNEL } from '../../preload'
import { showSettings } from './window'

const parser = new $RefParser()

export const getConfig = async () => {
  const schemaRaw = config.schema()
  const schema = await parser.dereference(schemaRaw as $RefParser.JSONSchema)

  return {
    config: config.read(),
    schema: schema,
  }
}

interface NormalizedPlugins {
  entities: Record<string, plugins.Plugin>
  ids: string[]
}

export const getPlugins = () => {
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
  ipcMain.handle(CHANNEL.SHOW_CONFIG_WINDOW, async () => {
    return showSettings()
  })

  ipcMain.handle(CHANNEL.READ_CONFIG, async () => {
    return getConfig()
  })

  ipcMain.handle(CHANNEL.READ_PLUGINS, async () => {
    return getPlugins()
  })

  ipcMain.handle(CHANNEL.LIST_AVAILABLE_PLUGINS, async () => {
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
}
