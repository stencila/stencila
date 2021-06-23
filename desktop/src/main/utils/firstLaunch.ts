import {
  getAppConfig,
  setAppConfig,
  UnprotectedStoreKeys,
} from '../store/handlers'

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
