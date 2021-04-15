import { ipcRenderer } from 'electron'
import fs from 'fs'
import Store from 'secure-electron-store'

// Create the electron store to be made available in the renderer process
const store = new Store()

export const rendererStore = store.preloadBindings(ipcRenderer, fs)
