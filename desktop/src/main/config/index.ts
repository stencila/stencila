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

export const getPlugins = () => plugins.list()

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
}
