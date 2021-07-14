import { createStore, ObservableMap } from '@stencil/store'
import { app } from 'electron'
import fs from 'fs'
import path from 'path'
import { AppConfigStore } from '../../preload/types'

const storeName = 'storeUnprotected.json'
const userDataPath = app.getPath('userData')
export const unprotectedStorePath = path.join(userDataPath, storeName)

export const readUnprotectedStore = (): AppConfigStore => {
  try {
    const contents = fs.readFileSync(unprotectedStorePath)
    return JSON.parse(contents.toString())
  } catch {
    // TODO: Log error re. possibly corrupted config
  }

  return {}
}

const defaultConfigStore: AppConfigStore = {
  REPORT_ERRORS: false,
}

export let unprotectedStore: ObservableMap<AppConfigStore>

export const writeUnprotectedStore = (store: AppConfigStore): void => {
  fs.writeFileSync(unprotectedStorePath, JSON.stringify(store))
}

export const resetUnprotectedStore = (): void => {
  fs.writeFileSync(unprotectedStorePath, JSON.stringify(defaultConfigStore))
}

export const initAppConfigStore = () => {
  let config = defaultConfigStore
  if (fs.existsSync(unprotectedStorePath)) {
    config = readUnprotectedStore()
  }

  unprotectedStore = createStore<AppConfigStore>(config)

  unprotectedStore.on('set', () => {
    writeUnprotectedStore(unprotectedStore.state)
  })

  return unprotectedStore
}
