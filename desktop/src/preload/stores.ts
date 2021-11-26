import { AppConfigStore, ConfigPaths } from './types'

export const UnprotectedStoreKeys: Record<
  keyof AppConfigStore,
  keyof AppConfigStore
> = {
  USER_ID: 'USER_ID',
  FIRST_LAUNCH: 'FIRST_LAUNCH',
} as const

export const GlobalConfigKeys = {
  EDITOR_LINE_WRAPPING: 'editors.lineWrapping',
  EDITOR_LINE_NUMBERS: 'editors.lineNumbers',
  EDITOR_NEW_FILE_SYNTAX: 'editors.defaultFormat',
  REPORT_ERRORS: 'telemetry.desktop.error_reports',
} as const

export const isGlobalConfigPath = (path: any): path is ConfigPaths => {
  return Object.values(GlobalConfigKeys).includes(path)
}

export const isUnprotectedStoreKey = (
  key: any
): key is keyof AppConfigStore => {
  return Object.keys(UnprotectedStoreKeys).includes(key)
}
