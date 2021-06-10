import { unprotectedStore, AppConfigStore, JSONValue } from './bootstrap'

export enum UnprotectedStoreKeys {
  REPORT_ERRORS = 'REPORT_ERRORS',
  FIRST_LAUNCH = 'FIRST_LAUNCH'
}

export const readAppConfig = () => {
  return unprotectedStore.state
}

export const getAppConfig = (key: UnprotectedStoreKeys) => {
  return unprotectedStore.get(key)
}

export const setAppConfig = (key: UnprotectedStoreKeys) => (
  value: JSONValue
) => {
  unprotectedStore.set(key, value)
}

export const updateAppConfig = (newStore: AppConfigStore) => {
  unprotectedStore.state = newStore
}
