import { app, BrowserWindow, ipcMain } from 'electron'
import fs from 'fs'
import Store from 'secure-electron-store'

export const initStore = (win: BrowserWindow) => {
  const store = new Store({
    path: app.getPath('userData'),
  })

  store.mainBindings(ipcMain, win, fs)

  return store
}
