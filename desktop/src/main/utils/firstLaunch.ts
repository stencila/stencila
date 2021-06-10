import { app } from 'electron'
import path from 'path'
import {
  getAppConfig,
  setAppConfig,
  UnprotectedStoreKeys
} from '../store/handlers'

const storeName = 'unprotected.json'
const userDataPath = app.getPath('userData')
export const unprotectedStorePath = path.join(userDataPath, storeName)

/**
 * Checks whether the app is being launched for the first time
 */
export const isFirstLaunch = (): boolean => {
  const config = getAppConfig(UnprotectedStoreKeys.FIRST_LAUNCH)
  return typeof config === 'boolean' ? config : true
}

export const setFirstLaunchState = (value: boolean) => {
  setAppConfig(UnprotectedStoreKeys.FIRST_LAUNCH)(value)
}
