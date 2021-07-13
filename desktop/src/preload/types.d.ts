import { IpcMainInvokeEvent } from 'electron'
import { JSONSchema7 } from 'json-schema'
import { Config, dispatch, Plugin, Result } from 'stencila'
import { Channel, CHANNEL, Handler } from './channels'
import { LogHandler } from './errors'
import { UnprotectedStoreKeys } from './stores'

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

type AnyFunction = (...args: any) => any

type Resultify<F> = F extends Result<any> ? F : Result<F>

/**
 * Type wrapper for defining the return types of calling a `invoke`/`dispatch` method.
 */
type InvokeResult<T> = Promise<Resultify<T>>

type InvokeType<C extends Channel, F extends AnyFunction> = {
  channel: C
  args: Parameters<F>
  result: InvokeResult<ReturnType<F>>
}

// -----------------------------------------------------------------------------

// Global
export type OpenLink = InvokeType<
  typeof CHANNEL.OPEN_LINK_IN_DEFAULT_BROWSER,
  (url: string) => void
>

export type CaptureError = InvokeType<
  typeof CHANNEL.CAPTURE_ERROR,
  (payload: LogHandler) => void
>

// Config
export type ReadConfig = InvokeType<
  typeof CHANNEL.READ_CONFIG,
  () => {
    config: Config
    schema: JSONSchema7
  }
>

export type ReadAppConfig = InvokeType<
  typeof CHANNEL.READ_APP_CONFIG,
  () => AppConfigStore
>

export type GetAppConfig = InvokeType<
  typeof CHANNEL.GET_APP_CONFIG,
  (key: UnprotectedStoreKeys) => InvokeResult<JSONValue>
>

export type SetAppConfig = InvokeType<
  typeof CHANNEL.SET_APP_CONFIG,
  (payload: { key: UnprotectedStoreKeys; value: JSONValue }) => void
>

// Plugins
export type PluginsRead = InvokeType<
  typeof CHANNEL.READ_PLUGINS,
  () => NormalizedPlugins
>

export type PluginsList = InvokeType<
  typeof CHANNEL.LIST_AVAILABLE_PLUGINS,
  () => NormalizedPlugins
>

export type PluginsInstall = InvokeType<
  typeof CHANNEL.INSTALL_PLUGIN,
  typeof dispatch.plugins.install
>

export type PluginsUninstall = InvokeType<
  typeof CHANNEL.UNINSTALL_PLUGIN,
  typeof dispatch.plugins.uninstall
>

export type PluginsUpgrade = InvokeType<
  typeof CHANNEL.UPGRADE_PLUGIN,
  typeof dispatch.plugins.upgrade
>

export type PluginsRefresh = InvokeType<
  typeof CHANNEL.REFRESH_PLUGINS,
  typeof dispatch.plugins.refresh
>

// Launcher
export type LauncherWindowOpen = InvokeType<
  typeof CHANNEL.OPEN_LAUNCHER_WINDOW,
  () => void
>
export type LauncherWindowClose = InvokeType<
  typeof CHANNEL.CLOSE_LAUNCHER_WINDOW,
  () => void
>

// Onboarding
export type OnboardingWindowOpen = InvokeType<
  typeof CHANNEL.OPEN_ONBOARDING_WINDOW,
  () => void
>
export type OnboardingWindowClose = InvokeType<
  typeof CHANNEL.CLOSE_ONBOARDING_WINDOW,
  () => void
>

// Projects
export type ProjectsWindowOpen = InvokeType<
  typeof CHANNEL.OPEN_PROJECT_WINDOW,
  (directoryPath: string) => void
>

export type ProjectsOpenUsingFilePicker = InvokeType<
  typeof CHANNEL.SELECT_PROJECT_DIR,
  () => void
>

export type ProjectsOpen = InvokeType<
  typeof CHANNEL.GET_PROJECT_FILES,
  typeof dispatch.projects.open
>

// Documents
export type DocumentsOpen = InvokeType<
  typeof CHANNEL.DOCUMENTS_OPEN,
  typeof dispatch.documents.open
>

export type DocumentsClose = InvokeType<
  typeof CHANNEL.CLOSE_DOCUMENT,
  typeof dispatch.documents.close
>

export type DocumentsCloseActive = InvokeType<
  typeof CHANNEL.CLOSE_ACTIVE_DOCUMENT,
  (path: string) => void
>

export type DocumentsPreview = InvokeType<
  typeof CHANNEL.GET_DOCUMENT_PREVIEW,
  typeof dispatch.documents.dump
>

export type DocumentsDump = InvokeType<
  typeof CHANNEL.GET_DOCUMENT_CONTENTS,
  typeof dispatch.documents.dump
>

export type DocumentsWrite = InvokeType<
  typeof CHANNEL.SAVE_DOCUMENT,
  typeof dispatch.documents.write
>

export type DocumentsUnsubscribe = InvokeType<
  typeof CHANNEL.UNSUBSCRIBE_DOCUMENT,
  typeof dispatch.documents.unsubscribe
>

type InvokeTypes =
  | OpenLink
  | CaptureError
  | ReadConfig
  | ReadAppConfig
  | GetAppConfig
  | SetAppConfig
  | PluginsRead
  | PluginsList
  | PluginsInstall
  | PluginsUninstall
  | PluginsUpgrade
  | PluginsRefresh
  | LauncherWindowOpen
  | LauncherWindowClose
  | OnboardingWindowOpen
  | OnboardingWindowClose
  | ProjectsWindowOpen
  | ProjectsOpenUsingFilePicker
  | ProjectsOpen
  | DocumentsOpen
  | DocumentsClose
  | DocumentsCloseActive
  | DocumentsPreview
  | DocumentsDump
  | DocumentsWrite
  | DocumentsUnsubscribe

// -----------------------------------------------------------------------------

type InvokeListener<F extends InvokeTypes> = (
  ipcEvent: IpcMainInvokeEvent,
  ...args: F['args']
) => F['result']

export type InvokeHandler<F extends InvokeTypes> = (
  channel: F['channel'],
  listener: InvokeListener<F>
) => void

// -----------------------------------------------------------------------------

interface Invoke {
  // Global
  invoke(
    channel: OpenLink['channel'],
    ...args: OpenLink['args']
  ): OpenLink['result']

  invoke(
    channel: CaptureError['channel'],
    ...args: CaptureError['args']
  ): CaptureError['result']

  // Config
  invoke(
    channel: ReadConfig['channel'],
    ...args: ReadConfig['args']
  ): ReadConfig['result']

  invoke(
    channel: ReadAppConfig['channel'],
    ...args: ReadAppConfig['args']
  ): ReadAppConfig['result']

  invoke(
    channel: GetAppConfig['channel'],
    ...args: GetAppConfig['args']
  ): GetAppConfig['result']

  invoke(
    channel: SetAppConfig['channel'],
    ...args: SetAppConfig['args']
  ): SetAppConfig['result']

  // Plugins
  invoke(
    channel: PluginsRead['channel'],
    ...args: PluginsRead['args']
  ): PluginsRead['result']

  invoke(
    channel: PluginsList['channel'],
    ...args: PluginsList['args']
  ): PluginsList['result']

  invoke(
    channel: PluginsInstall['channel'],
    ...args: PluginsInstall['args']
  ): PluginsInstall['result']

  invoke(
    channel: PluginsUninstall['channel'],
    ...args: PluginsUninstall['args']
  ): PluginsUninstall['result']

  invoke(
    channel: PluginsUpgrade['channel'],
    ...args: PluginsUpgrade['args']
  ): PluginsUpgrade['result']

  invoke(
    channel: PluginsRefresh['channel'],
    ...args: PluginsRefresh['args']
  ): PluginsRefresh['result']
  invoke(channel: PluginsRefresh['channel']): PluginsRefresh['result']

  // Launcher
  invoke(
    channel: LauncherWindowOpen['channel'],
    ...args: LauncherWindowOpen['args']
  ): LauncherWindowOpen['result']

  invoke(
    channel: LauncherWindowClose['channel'],
    ...args: LauncherWindowClose['args']
  ): LauncherWindowClose['result']

  // Onboarding
  invoke(
    channel: OnboardingWindowOpen['channel'],
    ...args: OnboardingWindowOpen['args']
  ): OnboardingWindowOpen['result']

  invoke(
    channel: OnboardingWindowClose['channel'],
    ...args: OnboardingWindowClose['args']
  ): OnboardingWindowClose['result']

  // Projects
  invoke(
    channel: ProjectsWindowOpen['channel'],
    ...args: ProjectsWindowOpen['args']
  ): ProjectsWindowOpen['result']

  invoke(
    channel: ProjectsOpenUsingFilePicker['channel'],
    ...args: ProjectsOpenUsingFilePicker['args']
  ): ProjectsOpenUsingFilePicker['result']

  invoke(
    channel: ProjectsOpen['channel'],
    ...args: ProjectsOpen['args']
  ): ProjectsOpen['result']

  // Documents
  invoke(
    channel: DocumentsOpen['channel'],
    ...args: DocumentsOpen['args']
  ): DocumentsOpen['result']

  invoke(
    channel: DocumentsClose['channel'],
    ...args: DocumentsClose['args']
  ): DocumentsClose['result']

  invoke(
    channel: DocumentsCloseActive['channel'],
    ...args: DocumentsCloseActive['args']
  ): DocumentsCloseActive['result']

  invoke(
    channel: DocumentsPreview['channel'],
    ...args: DocumentsPreview['args']
  ): DocumentsPreview['result']

  invoke(
    channel: DocumentsDump['channel'],
    ...args: DocumentsDump['args']
  ): DocumentsDump['result']

  invoke(
    channel: DocumentsWrite['channel'],
    ...args: DocumentsWrite['args']
  ): DocumentsWrite['result']

  invoke(
    channel: DocumentsUnsubscribe['channel'],
    ...args: DocumentsUnsubscribe['args']
  ): DocumentsUnsubscribe['result']
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
