import { webContents } from 'electron'
import { CHANNEL } from '../../preload/channels'
import { UnprotectedStoreKeys } from '../../preload/stores'
import { AppConfigStore } from '../../preload/types'
import { defaultConfigStore, unprotectedStore } from './bootstrap'

export const readAppConfig = () => {
  return unprotectedStore.state
}

export const getAppConfig = <K extends UnprotectedStoreKeys>(
  key: K
): AppConfigStore[K] => {
  return unprotectedStore.get(key)
}

export const setAppConfig =
  <K extends UnprotectedStoreKeys>(key: K) =>
  (value: AppConfigStore[K]) => {
    unprotectedStore.set(key, value)

    // Inform all open windows of the configuration change.
    // Each window should register a listener for the `CHANNEL.CONFIG_APP_SET` and
    // react as needed to the changes.
    webContents.getAllWebContents().forEach((wc) => {
      wc.send(CHANNEL.CONFIG_APP_SET, { key, value })
    })
  }

export const updateAppConfig = (newStore: AppConfigStore) => {
  unprotectedStore.state = newStore
}

export const isReportErrorsEnabled = (): boolean => {
  const value = getAppConfig(UnprotectedStoreKeys.REPORT_ERRORS)
  return typeof value === 'boolean' ? value : value === 'true'
}

export const isLineNumbersEnabled = (): boolean => {
  const value = getAppConfig(UnprotectedStoreKeys.EDITOR_LINE_NUMBERS)
  return value ?? defaultConfigStore.EDITOR_LINE_NUMBERS
}

export const toggleLineNumbers = () => {
  setAppConfig(UnprotectedStoreKeys.EDITOR_LINE_NUMBERS)(
    !isLineNumbersEnabled()
  )
}

export const isLineWrappingEnabled = (): boolean => {
  const value = getAppConfig(UnprotectedStoreKeys.EDITOR_LINE_WRAPPING)
  return value ?? defaultConfigStore.EDITOR_LINE_WRAPPING
}

export const toggleLineWrapping = () => {
  setAppConfig(UnprotectedStoreKeys.EDITOR_LINE_WRAPPING)(
    !isLineWrappingEnabled()
  )
}
