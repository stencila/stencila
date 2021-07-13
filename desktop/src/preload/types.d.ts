import {
  Config,
  dispatch,
  Document,
  documents,
  Plugin,
  plugins,
  Project,
  Result,
} from 'stencila'
import { Channel, CHANNEL, Handler } from './channels'
import { UnprotectedStoreKeys } from './stores'
import { JSONSchema7 } from 'json-schema'

type EntityId = number | string

export type JSONValue =
  | string
  | number
  | boolean
  | null
  | JSONValue[]
  | { [key: string]: JSONValue }

export interface AppConfigStore {
  [key: string]: JSONValue
}

export interface NormalizedPlugins {
  entities: Record<string, Plugin>
  ids: string[]
}

/**
 * Type wrapper for defining the return types of calling a `invoke`/`dispatch` method.
 */
type InvokeResult<T> = Promise<Result<T>>

interface Invoke {
  // Global
  invoke(
    channel: typeof CHANNEL.OPEN_LINK_IN_DEFAULT_BROWSER,
    url: string
  ): InvokeResult<void>

  // Config
  invoke(channel: typeof CHANNEL.READ_CONFIG): InvokeResult<{
    config: Config
    schema: JSONSchema7
  }>

  invoke(channel: typeof CHANNEL.READ_APP_CONFIG): InvokeResult<AppConfigStore>

  invoke(
    channel: typeof CHANNEL.GET_APP_CONFIG,
    key: UnprotectedStoreKeys
  ): InvokeResult<JSONValue>

  invoke(
    channel: typeof CHANNEL.SET_APP_CONFIG,
    payload: { key: UnprotectedStoreKeys; value: JSONValue }
  ): InvokeResult<void>

  // Plugins
  invoke(channel: typeof CHANNEL.READ_PLUGINS): InvokeResult<NormalizedPlugins>

  invoke(
    channel: typeof CHANNEL.LIST_AVAILABLE_PLUGINS
  ): InvokeResult<NormalizedPlugins>

  invoke(
    channel: typeof CHANNEL.INSTALL_PLUGIN,
    ...args: Parameters<typeof plugins.install>
  ): InvokeResult<Plugin[]>

  invoke(
    channel: typeof CHANNEL.UNINSTALL_PLUGIN,
    ...args: Parameters<typeof plugins.uninstall>
  ): InvokeResult<Plugin[]>

  invoke(
    channel: typeof CHANNEL.UPGRADE_PLUGIN,
    ...args: Parameters<typeof plugins.uninstall>
  ): InvokeResult<Plugin[]>

  invoke(
    channel: typeof CHANNEL.REFRESH_PLUGINS,
    ...args: Parameters<typeof plugins.refresh>
  ): InvokeResult<Plugin[]>
  invoke(channel: typeof CHANNEL.REFRESH_PLUGINS): InvokeResult<Plugin[]>

  // Launcher
  invoke(channel: typeof CHANNEL.OPEN_LAUNCHER_WINDOW): InvokeResult<void>
  invoke(channel: typeof CHANNEL.CLOSE_LAUNCHER_WINDOW): InvokeResult<void>

  // Onboarding
  invoke(channel: typeof CHANNEL.OPEN_ONBOARDING_WINDOW): InvokeResult<void>
  invoke(channel: typeof CHANNEL.CLOSE_ONBOARDING_WINDOW): InvokeResult<void>

  // Projects
  invoke(
    channel: typeof CHANNEL.OPEN_PROJECT,
    directoryPath: string
  ): InvokeResult<void>

  invoke(channel: typeof CHANNEL.SELECT_PROJECT_DIR): InvokeResult<void>

  invoke(
    channel: typeof CHANNEL.GET_PROJECT_FILES,
    path: string
  ): InvokeResult<Project>

  // Documents
  invoke(
    channel: typeof CHANNEL.DOCUMENTS_OPEN,
    ...args: Parameters<typeof dispatch.documents.open>
  ): InvokeResult<Document>

  invoke(
    channel: typeof CHANNEL.CLOSE_DOCUMENT,
    ...args: Parameters<typeof documents.close>
  ): InvokeResult<void>

  invoke(
    channel: typeof CHANNEL.CLOSE_ACTIVE_DOCUMENT,
    path: string
  ): InvokeResult<void>

  invoke(
    channel: typeof CHANNEL.GET_DOCUMENT_PREVIEW,
    documentId: EntityId
  ): InvokeResult<string>

  invoke(
    channel: typeof CHANNEL.GET_DOCUMENT_CONTENTS,
    documentId: EntityId
  ): InvokeResult<string>

  invoke(
    channel: typeof CHANNEL.SAVE_DOCUMENT,
    payload: {
      documentId: EntityId
      content: string
    }
  ): InvokeResult<string>

  invoke(
    channel: typeof CHANNEL.UNSUBSCRIBE_DOCUMENT,
    payload: {
      documentId: EntityId
      topics: string[]
    }
  ): InvokeResult<void>
}

export interface IpcRendererAPI extends Invoke {
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
