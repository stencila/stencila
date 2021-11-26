import { createStore, ObservableMap } from '@stencil/store'
import { app } from 'electron'
import fs from 'fs'
import path from 'path'
import { config } from 'stencila'
import { AppConfigStore, CombinedConfig } from '../../preload/types'

const storeName = 'storeUnprotected.json'
const userDataPath = app.getPath('userData')
export const unprotectedStorePath = path.join(userDataPath, storeName)

export const defaultConfigStore: AppConfigStore = {}

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
  ) as (keyof AppConfigStore)[]
  const currentStoreKeys = Object.keys(currentConfig)

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
  const combinedConfig: CombinedConfig = {
    app: defaultConfigStore,
    global: config.get(),
  }

  if (fs.existsSync(unprotectedStorePath)) {
    combinedConfig.app = readUnprotectedStore()
  }

  const store = createStore<CombinedConfig>(combinedConfig)

  store.on('set', () => {
    writeUnprotectedStore(store.state.app)
  })

  setMissingConfigValues(combinedConfig.app)

  return store
}

export let unprotectedStore: ObservableMap<CombinedConfig> =
  initAppConfigStore()
