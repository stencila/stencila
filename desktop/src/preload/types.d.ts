import { EntityId } from '@reduxjs/toolkit'
import { JSONSchema7 } from 'json-schema'
import { Config, documents, Plugin, plugins, Project, Result } from 'stencila'
import { AppConfigStore, JSONValue } from '../main/store/bootstrap'
import { Channel, CHANNEL, Handler } from './channels'
import { UnprotectedStoreKeys } from './stores'

export interface NormalizedPlugins {
  entities: Record<string, Plugin>
  ids: string[]
}

type InvokeResult<R extends (...args: any) => any> = Promise<
  Result<ReturnType<R>>
>

interface Invoke {
  // Global
  invoke(
    channel: typeof CHANNEL.OPEN_LINK_IN_DEFAULT_BROWSER,
    url: string
  ): Promise<void>

  // Config
  invoke(channel: typeof CHANNEL.OPEN_CONFIG_WINDOW): Promise<void>

  invoke(channel: typeof CHANNEL.READ_CONFIG): Promise<{
    config: Config
    schema: JSONSchema7
  }>

  invoke(channel: typeof CHANNEL.READ_APP_CONFIG): Promise<AppConfigStore>

  invoke(
    channel: typeof CHANNEL.GET_APP_CONFIG,
    key: UnprotectedStoreKeys
  ): Promise<JSONValue>

  invoke(
    channel: typeof CHANNEL.SET_APP_CONFIG,
    payload: { key: UnprotectedStoreKeys; value: JSONValue }
  ): Promise<void>

  // Plugins
  invoke(channel: typeof CHANNEL.READ_PLUGINS): Promise<NormalizedPlugins>

  invoke(
    channel: typeof CHANNEL.LIST_AVAILABLE_PLUGINS
  ): Promise<NormalizedPlugins>

  invoke(
    channel: typeof CHANNEL.INSTALL_PLUGIN,
    ...args: Parameters<typeof plugins.install>
  ): Promise<Plugin[]>

  invoke(
    channel: typeof CHANNEL.UNINSTALL_PLUGIN,
    ...args: Parameters<typeof plugins.uninstall>
  ): Promise<Plugin[]>

  invoke(
    channel: typeof CHANNEL.UPGRADE_PLUGIN,
    ...args: Parameters<typeof plugins.uninstall>
  ): Promise<Plugin[]>

  invoke(
    channel: typeof CHANNEL.REFRESH_PLUGINS,
    ...args: Parameters<typeof plugins.refresh>
  ): Promise<Plugin[]>
  invoke(channel: typeof CHANNEL.REFRESH_PLUGINS): Promise<Plugin[]>

  // Launcher
  invoke(channel: typeof CHANNEL.OPEN_LAUNCHER_WINDOW): Promise<void>
  invoke(channel: typeof CHANNEL.CLOSE_LAUNCHER_WINDOW): Promise<void>

  // Onboarding
  invoke(channel: typeof CHANNEL.OPEN_ONBOARDING_WINDOW): Promise<void>
  invoke(channel: typeof CHANNEL.CLOSE_ONBOARDING_WINDOW): Promise<void>

  // Projects
  invoke(channel: typeof CHANNEL.SHOW_PROJECT_WINDOW): Promise<void>

  invoke(
    channel: typeof CHANNEL.OPEN_PROJECT,
    directoryPath: string
  ): Promise<void>

  invoke(channel: typeof CHANNEL.SELECT_PROJECT_DIR): Promise<void>

  invoke(
    channel: typeof CHANNEL.GET_PROJECT_FILES,
    path: string
  ): Promise<Project>

  // Documents
  invoke(
    channel: typeof CHANNEL.DOCUMENTS_OPEN,
    ...args: Parameters<typeof documents.open>
  ): InvokeResult<typeof documents.open>

  invoke(
    channel: typeof CHANNEL.CLOSE_DOCUMENT,
    ...args: Parameters<typeof documents.close>
  ): Promise<ReturnType<typeof documents.close>>

  invoke(
    channel: typeof CHANNEL.CLOSE_ACTIVE_DOCUMENT,
    path: string
  ): Promise<void>

  invoke(
    channel: typeof CHANNEL.GET_DOCUMENT_PREVIEW,
    documentId: EntityId
  ): Promise<string>

  invoke(
    channel: typeof CHANNEL.GET_DOCUMENT_CONTENTS,
    documentId: EntityId
  ): Promise<string>

  invoke(
    channel: typeof CHANNEL.SAVE_DOCUMENT,
    payload: {
      documentId: EntityId
      content: string
    }
  ): Promise<void>

  invoke(
    channel: typeof CHANNEL.UNSUBSCRIBE_DOCUMENT,
    payload: {
      documentId: EntityId
      topics: string[]
    }
  ): Promise<void>
}

interface IpcRendererAPI extends Invoke {
  send(channel: Channel, ...args: unknown[]): void
  receive: (channel: Channel, func: Handler) => void
  remove: (channel: Channel, func: Handler) => void
  removeAll: (channel: Channel) => void
}

declare global {
  interface Window {
    api: IpcRendererAPI
  }
}
