declare module 'secure-electron-store' {
  import { BrowserWindow, IpcMain, IpcRenderer } from 'electron'
  import fs from 'fs'

  type Options = Partial<{
    debug: boolean
    minify: boolean
    encrypt: boolean
    passkey: string
    path: string
    unprotectedPath: string
    filename: string
    unprotectedFilename: string
    extension: string
    reset: boolean
  }>

  class Store {
    constructor(options?: Options)

    public mainBindings(
      ipcMain: IpcMain,
      win: BrowserWindow,
      nodeFs: typeof fs
    ): Store

    public preloadBindings(ipcRenderer: IpcRenderer, nodeFs: typeof fs): Store
  }

  export default Store
}
