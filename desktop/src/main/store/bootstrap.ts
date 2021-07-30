import { createStore, ObservableMap } from '@stencil/store'
import { app } from 'electron'
import fs from 'fs'
import path from 'path'
import { UnprotectedStoreKeys } from '../../preload/stores'
import { AppConfigStore } from '../../preload/types'

const storeName = 'storeUnprotected.json'
const userDataPath = app.getPath('userData')
export const unprotectedStorePath = path.join(userDataPath, storeName)

export const defaultConfigStore: AppConfigStore = {
  REPORT_ERRORS: false,
  EDITOR_LINE_NUMBERS: true,
  EDITOR_LINE_WRAPPING: true,
  EDITOR_NEW_FILE_SYNTAX: 'md',
}

export const readUnprotectedStore = (): AppConfigStore => {
  try {
    const contents = fs.readFileSync(unprotectedStorePath)
    return JSON.parse(contents.toString())
  } catch {
    // TODO: Log error re. possibly corrupted config
  }

  return defaultConfigStore
}

export const writeUnprotectedStore = (store: AppConfigStore): void => {
  fs.writeFileSync(unprotectedStorePath, JSON.stringify(store))
}

export const resetUnprotectedStore = (): void => {
  fs.writeFileSync(unprotectedStorePath, JSON.stringify(defaultConfigStore))
}

/**
 * If there are any default configuration settings missing from the user settings
 * file on disk, patch the settings file with the default values.
 */
const setMissingConfigValues = (currentConfig: AppConfigStore) => {
  const defaultStoreKeys = Object.keys(
    defaultConfigStore
  ) as UnprotectedStoreKeys[]
  const currentStoreKeys = Object.keys(currentConfig) as UnprotectedStoreKeys[]

  const missingConfig = defaultStoreKeys.reduce(
    (config: Partial<AppConfigStore>, key) =>
      currentStoreKeys.includes(key)
        ? config
        : { ...config, [key]: defaultConfigStore[key] },
    {}
  )

  if (Object.keys(missingConfig).length > 0) {
    writeUnprotectedStore({
      ...currentConfig,
      ...missingConfig,
    })
  }
}

const initAppConfigStore = () => {
  let config = defaultConfigStore
  if (fs.existsSync(unprotectedStorePath)) {
    config = readUnprotectedStore()
  }

  const store = createStore<AppConfigStore>(config)

  store.on('set', () => {
    writeUnprotectedStore(store.state)
  })

  setMissingConfigValues(config)

  return store
}

export let unprotectedStore: ObservableMap<AppConfigStore> =
  initAppConfigStore()
