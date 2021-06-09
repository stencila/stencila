import { ipcMain } from 'electron'
import { CHANNEL } from '../../preload'
import { JSONValue } from './bootstrap'
import {
  getAppConfig,
  readAppConfig,
  setAppConfig,
  UnprotectedStoreKeys,
} from './handlers'

export const registerAppConfigStoreHandlers = () => {
  ipcMain.handle(CHANNEL.READ_APP_CONFIG, () => {
    const config = readAppConfig()
    return { ...config }
  })

  ipcMain.handle(
    CHANNEL.GET_APP_CONFIG,
    (_event, key: UnprotectedStoreKeys) => {
      return getAppConfig(key)
    }
  )

  ipcMain.handle(
    CHANNEL.SET_APP_CONFIG,
    (
      _event,
      { key, value }: { key: UnprotectedStoreKeys; value: JSONValue }
    ) => {
      return setAppConfig(key)(value)
    }
  )
}
