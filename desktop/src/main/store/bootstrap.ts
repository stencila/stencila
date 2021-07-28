import { createStore, ObservableMap } from '@stencil/store'
import { app } from 'electron'
import fs from 'fs'
import path from 'path'
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

const initAppConfigStore = () => {
  let config = defaultConfigStore
  if (fs.existsSync(unprotectedStorePath)) {
    config = readUnprotectedStore()
  }

  const store = createStore<AppConfigStore>(config)

  store.on('set', () => {
    writeUnprotectedStore(store.state)
  })

  return store
}

export let unprotectedStore: ObservableMap<AppConfigStore> =
  initAppConfigStore()

export const writeUnprotectedStore = (store: AppConfigStore): void => {
  fs.writeFileSync(unprotectedStorePath, JSON.stringify(store))
}

export const resetUnprotectedStore = (): void => {
  fs.writeFileSync(unprotectedStorePath, JSON.stringify(defaultConfigStore))
}
