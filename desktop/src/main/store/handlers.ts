import { UnprotectedStoreKeys } from '../../preload/stores'
import { AppConfigStore, JSONValue } from '../../preload/types'
import { unprotectedStore } from './bootstrap'

export const readAppConfig = () => {
  return unprotectedStore.state
}

export const getAppConfig = (key: UnprotectedStoreKeys) => {
  return unprotectedStore.get(key)
}

export const setAppConfig =
  (key: UnprotectedStoreKeys) => (value: JSONValue) => {
    unprotectedStore.set(key, value)
  }

export const updateAppConfig = (newStore: AppConfigStore) => {
  unprotectedStore.state = newStore
}

export const isReportErrorsEnabled = (): boolean => {
  const value = getAppConfig(UnprotectedStoreKeys.REPORT_ERRORS)
  return typeof value === 'boolean' ? value : value === 'true'
}
