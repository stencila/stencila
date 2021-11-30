import { config } from 'stencila'
import { CHANNEL } from '../../preload/channels'
import { GlobalConfigKeys, isGlobalConfigPath } from '../../preload/stores'
import {
  AppConfigStore,
  CombinedConfig,
  ConfigPaths,
} from '../../preload/types'
import { unprotectedStore } from '../store/bootstrap'
import { sendToAllWindows } from '../utils/ipc'

/**
 * Update the shared (between CLI and Desktop) Stencila configuration
 */
const setGlobalConfig = (key: ConfigPaths, value: string): void => {
  config.setProperty(key, value)
  unprotectedStore.set('global', config.get())
}

/**
 * Update the Desktop app specific configuration
 */
const setAppConfig =
  <K extends keyof AppConfigStore>(key: K) =>
  (value: AppConfigStore[K]) => {
    unprotectedStore.state.app[key] = value
  }

export const getConfig = (): CombinedConfig => {
  return {
    app: unprotectedStore.get('app'),
    global: unprotectedStore.get('global'),
  }
}

/**
 * Update the specified configuration and inform all open windows of the configuration change.
 * Each window should register a listener for the `CHANNEL.CONFIG_APP_SET` and react as needed to the changes.
 */
export const setConfig = <K extends ConfigPaths | keyof AppConfigStore>(
  key: K,
  value: K extends ConfigPaths
    ? string
    : K extends keyof AppConfigStore
    ? AppConfigStore[K]
    : never
): void => {
  if (isGlobalConfigPath(key)) {
    if (value !== undefined) {
      setGlobalConfig(key, value.toString())
      sendToAllWindows(CHANNEL.CONFIG_SET, {
        key: 'global',
        value: config.get(),
      })
    }
  } else {
    setAppConfig(key)(value)
    sendToAllWindows(CHANNEL.CONFIG_SET, {
      key: 'app',
      value: unprotectedStore.state.app,
    })
  }
}

export const updateAppConfig = (newStore: AppConfigStore): void => {
  unprotectedStore.state.app = newStore
}

export const isReportErrorsEnabled = (): boolean => {
  const value = getConfig().global.telemetry?.desktop?.error_reports
  return typeof value === 'boolean' ? value : value === 'true'
}

export const isLineNumbersEnabled = (): boolean => {
  return getConfig().global.editors?.lineNumbers ?? true
}

export const toggleLineNumbers = (): void => {
  setConfig(
    // @ts-expect-error
    GlobalConfigKeys.EDITOR_LINE_NUMBERS,
    (!isLineNumbersEnabled()).toString()
  )
}

export const isLineWrappingEnabled = (): boolean => {
  return getConfig().global.editors?.lineWrapping ?? true
}

export const toggleLineWrapping = (): void => {
  setConfig(
    // @ts-expect-error
    GlobalConfigKeys.EDITOR_LINE_WRAPPING,
    (!isLineWrappingEnabled()).toString()
  )
}
