import { IpcMainInvokeEvent } from 'electron'
import log, { LogMessage } from 'electron-log'
import { Config, dispatch, Plugin, Result } from 'stencila'
import { Channel, CHANNEL, Handler } from './channels'

type EntityId = number | string

export type JSONValue =
  | string
  | number
  | boolean
  | null
  | JSONValue[]
  | { [key: string]: JSONValue }

export interface AppConfigStore {
  USER_ID?: string
  FIRST_LAUNCH?: boolean | undefined
}

// Build up a path string accessor
// Based on https://stackoverflow.com/a/67609485/5686014
// Used to generate the types needed for setting configuration parameters via the CLI interface
type JoinPaths<K, P> = K extends string | number
  ? P extends string | number
    ? `${K}${'' extends P ? '' : '.'}${P}`
    : never
  : never

type PrevPath = [never, 0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, ...0[]]

type ObjectPaths<T, D extends number = 10> = [D] extends [never]
  ? never
  : T extends Record<string, unknown>
  ? {
      [K in keyof T]-?: K extends string | number
        ? `${K}` | JoinPaths<K, ObjectPaths<T[K], PrevPath[D]>>
        : never
    }[keyof T]
  : ''

export type ConfigPaths = ObjectPaths<Config>

export interface CombinedConfig {
  app: AppConfigStore
  global: Config
}

export interface NormalizedPlugins {
  entities: Record<string, Plugin>
  ids: string[]
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type AnyFunction = (...args: any) => any

// eslint-disable-next-line @typescript-eslint/no-explicit-any
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

export type GetAppVersion = InvokeType<
  typeof CHANNEL.GET_APP_VERSION,
  () => string
>

// Logs
export type LogsWindowOpen = InvokeType<
  typeof CHANNEL.LOGS_WINDOW_OPEN,
  () => void
>

export type LogsGet = InvokeType<typeof CHANNEL.LOGS_GET, () => LogMessage[]>

// Config
export type ConfigWindowOpen = InvokeType<
  typeof CHANNEL.CONFIG_WINDOW_OPEN,
  () => void
>

export type ConfigGetAll = InvokeType<
  typeof CHANNEL.CONFIG_GET_ALL,
  () => CombinedConfig
>

export type ConfigSet = InvokeType<
  typeof CHANNEL.CONFIG_SET,
  <K extends ConfigPaths | keyof AppConfigStore>(payload: {
    key: K
    value: K extends ConfigPaths
      ? string
      : K extends keyof AppConfigStore
      ? AppConfigStore[K]
      : never
  }) => void
>

// Plugins
export type PluginsList = InvokeType<
  typeof CHANNEL.PLUGINS_LIST,
  () => NormalizedPlugins
>

export type PluginsInstall = InvokeType<
  typeof CHANNEL.PLUGINS_INSTALL,
  typeof dispatch.plugins.install
>

export type PluginsUninstall = InvokeType<
  typeof CHANNEL.PLUGINS_UNINSTALL,
  typeof dispatch.plugins.uninstall
>

export type PluginsUpgrade = InvokeType<
  typeof CHANNEL.PLUGIN_UPGRADE,
  typeof dispatch.plugins.upgrade
>

export type PluginsRefresh = InvokeType<
  typeof CHANNEL.PLUGINS_REFRESH,
  typeof dispatch.plugins.refresh
>

// Launcher
export type LauncherWindowOpen = InvokeType<
  typeof CHANNEL.LAUNCHER_WINDOW_OPEN,
  () => void
>
export type LauncherWindowClose = InvokeType<
  typeof CHANNEL.LAUNCHER_WINDOW_CLOSE,
  () => void
>

// Onboarding
export type OnboardingWindowOpen = InvokeType<
  typeof CHANNEL.ONBOARDING_WINDOW_OPEN,
  () => void
>
export type OnboardingWindowClose = InvokeType<
  typeof CHANNEL.ONBOARDING_WINDOW_CLOSE,
  () => void
>

// Projects
export type ProjectsWindowOpen = InvokeType<
  typeof CHANNEL.PROJECTS_WINDOW_OPEN,
  (directoryPath: string) => void
>

export type ProjectsOpenUsingFilePicker = InvokeType<
  typeof CHANNEL.PROJECTS_OPEN_FROM_FILE_PICKER,
  () => {
    canceled: boolean
  }
>

export type ProjectsNew = InvokeType<typeof CHANNEL.PROJECTS_NEW, () => void>

export type ProjectsOpen = InvokeType<
  typeof CHANNEL.PROJECTS_OPEN,
  typeof dispatch.projects.open
>

export type ProjectsWrite = InvokeType<
  typeof CHANNEL.PROJECTS_WRITE,
  typeof dispatch.projects.write
>

export type ProjectsGraph = InvokeType<
  typeof CHANNEL.PROJECTS_GRAPH,
  typeof dispatch.projects.graph
>

export type ProjectsUnsubscribe = InvokeType<
  typeof CHANNEL.PROJECTS_UNSUBSCRIBE,
  typeof dispatch.projects.unsubscribe
>

export type ProjectsServerStart = InvokeType<
  typeof CHANNEL.PROJECTS_SERVER_START,
  typeof dispatch.server.serve
>

export type ProjectsKernelsLanguages = InvokeType<
  typeof CHANNEL.PROJECTS_KERNELS_LANGUAGES,
  typeof dispatch.kernels.languages
>

// Documents
export type DocumentsOpen = InvokeType<
  typeof CHANNEL.DOCUMENTS_OPEN,
  typeof dispatch.documents.open
>

export type DocumentsAlter = InvokeType<
  typeof CHANNEL.DOCUMENTS_ALTER,
  typeof dispatch.documents.alter
>

export type DocumentsCreate = InvokeType<
  typeof CHANNEL.DOCUMENTS_CREATE,
  typeof dispatch.documents.create
>

export type DocumentsCreateFilePath = InvokeType<
  typeof CHANNEL.DOCUMENTS_CREATE_FILE_PATH,
  () =>
    | {
        filePath: string
        canceled: false
      }
    | { canceled: true }
>

export type DocumentsClose = InvokeType<
  typeof CHANNEL.DOCUMENTS_CLOSE,
  typeof dispatch.documents.close
>

export type DocumentsCloseActive = InvokeType<
  typeof CHANNEL.DOCUMENTS_CLOSE_ACTIVE,
  (path: string) => void
>

export type DocumentsPreview = InvokeType<
  typeof CHANNEL.DOCUMENTS_PREVIEW,
  typeof dispatch.documents.dump
>

export type DocumentsDump = InvokeType<
  typeof CHANNEL.DOCUMENTS_DUMP,
  typeof dispatch.documents.dump
>

export type DocumentsLoad = InvokeType<
  typeof CHANNEL.DOCUMENTS_LOAD,
  typeof dispatch.documents.load
>

export type DocumentsGet = InvokeType<
  typeof CHANNEL.DOCUMENTS_GET,
  typeof dispatch.documents.get
>

export type DocumentsWrite = InvokeType<
  typeof CHANNEL.DOCUMENTS_WRITE,
  typeof dispatch.documents.write
>

export type DocumentsWriteAs = InvokeType<
  typeof CHANNEL.DOCUMENTS_WRITE_AS,
  | typeof dispatch.documents.writeAs
  | ((...args: Parameters<typeof dispatch.documents.writeAs>) => null)
>

export type DocumentsUnsubscribe = InvokeType<
  typeof CHANNEL.DOCUMENTS_UNSUBSCRIBE,
  typeof dispatch.documents.unsubscribe
>

type InvokeTypes =
  | OpenLink
  | GetAppVersion
  | LogsWindowOpen
  | LogsGet
  | ConfigWindowOpen
  | ConfigGetAll
  | ConfigSet
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
  | ProjectsNew
  | ProjectsOpen
  | ProjectsWrite
  | ProjectsGraph
  | ProjectsServerStart
  | ProjectsUnsubscribe
  | ProjectsKernelsLanguages
  | DocumentsOpen
  | DocumentsAlter
  | DocumentsCreate
  | DocumentsCreateFilePath
  | DocumentsClose
  | DocumentsCloseActive
  | DocumentsPreview
  | DocumentsDump
  | DocumentsLoad
  | DocumentsGet
  | DocumentsWrite
  | DocumentsWriteAs
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
    channel: GetAppVersion['channel'],
    ...args: GetAppVersion['args']
  ): GetAppVersion['result']

  // Logs
  invoke(
    channel: LogsWindowOpen['channel'],
    ...args: LogsWindowOpen['args']
  ): LogsWindowOpen['result']

  invoke(
    channel: LogsGet['channel'],
    ...args: LogsGet['args']
  ): LogsGet['result']

  // Config
  invoke(
    channel: ConfigWindowOpen['channel'],
    ...args: ConfigWindowOpen['args']
  ): ConfigWindowOpen['result']

  invoke(
    channel: ConfigGetAll['channel'],
    ...args: ConfigGetAll['args']
  ): ConfigGetAll['result']

  invoke(
    channel: ConfigSet['channel'],
    ...args: ConfigSet['args']
  ): ConfigSet['result']

  // Plugins
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
    channel: ProjectsNew['channel'],
    ...args: ProjectsNew['args']
  ): ProjectsNew['result']

  invoke(
    channel: ProjectsOpen['channel'],
    ...args: ProjectsOpen['args']
  ): ProjectsOpen['result']

  invoke(
    channel: ProjectsWrite['channel'],
    ...args: ProjectsWrite['args']
  ): ProjectsWrite['result']

  invoke(
    channel: ProjectsGraph['channel'],
    ...args: ProjectsGraph['args']
  ): ProjectsGraph['result']

  invoke(
    channel: ProjectsUnsubscribe['channel'],
    ...args: ProjectsUnsubscribe['args']
  ): ProjectsUnsubscribe['result']

  invoke(
    channel: ProjectsServerStart['channel'],
    ...args: ProjectsServerStart['args']
  ): ProjectsServerStart['result']

  invoke(
    channel: ProjectsKernelsLanguages['channel'],
    ...args: ProjectsKernelsLanguages['args']
  ): ProjectsKernelsLanguages['result']

  // Documents
  invoke(
    channel: DocumentsOpen['channel'],
    ...args: DocumentsOpen['args']
  ): DocumentsOpen['result']

  invoke(
    channel: DocumentsAlter['channel'],
    ...args: DocumentsAlter['args']
  ): DocumentsAlter['result']

  invoke(
    channel: DocumentsCreate['channel'],
    ...args: DocumentsCreate['args']
  ): DocumentsCreate['result']

  invoke(
    channel: DocumentsCreateFilePath['channel'],
    ...args: DocumentsCreateFilePath['args']
  ): DocumentsCreateFilePath['result']

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
    channel: DocumentsLoad['channel'],
    ...args: DocumentsLoad['args']
  ): DocumentsLoad['result']

  invoke(
    channel: DocumentsGet['channel'],
    ...args: DocumentsGet['args']
  ): DocumentsGet['result']

  invoke(
    channel: DocumentsWrite['channel'],
    ...args: DocumentsWrite['args']
  ): DocumentsWrite['result']

  invoke(
    channel: DocumentsWriteAs['channel'],
    ...args: DocumentsWriteAs['args']
  ): DocumentsWriteAs['result']

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
  log: typeof log
}

declare global {
  interface Window {
    api: IpcRendererAPI
  }
}
