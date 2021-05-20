import { app, BrowserWindow, ipcMain, protocol } from 'electron'
import { debug } from './debug'
import { createWindow } from './app/window'
import { main } from './main'
import { requestHandler, scheme } from './main/app-protocol'
import { initStore } from './main/store/bootstrap'

let store: ReturnType<typeof initStore>

// Handle creating/removing shortcuts on Windows when installing/uninstalling.
if (require('electron-squirrel-startup')) {
  app.quit()
}

if (process.platform === 'linux') {
  // This is necessary to avoid UI rendering glitches on Ubuntu, and possibly other Linux distributions.
  // TODO: Investigate the root cause and see if the OS targeting can be reduced to only apply to
  // specific OS distributions where needed.
  app.disableHardwareAcceleration()
}

const createMainWindow = (): void => {
  /* eng-disable PROTOCOL_HANDLER_JS_CHECK */
  protocol.registerBufferProtocol(scheme, requestHandler)

  const mainWindowUrl = '/'
  const mainWindow = createWindow(mainWindowUrl)

  store = initStore(mainWindow)
}

protocol.registerSchemesAsPrivileged([
  {
    scheme: scheme,
    privileges: {
      standard: true,
      secure: true,
    },
  },
])

if (process.env.NODE_ENV === 'development') {
  debug()
}

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
app.on('ready', createMainWindow)

// Quit when all windows are closed, except on macOS. There, it's common
// for applications and their menu bar to stay active until the user quits
// explicitly with Cmd + Q.
app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit()
  } else {
    store.clearMainBindings(ipcMain)
  }
})

app.on('activate', () => {
  // On OS X it's common to re-create a window in the app when the
  // dock icon is clicked and there are no other windows open.
  if (BrowserWindow.getAllWindows().length === 0) {
    createMainWindow()
  }
})

// In this file you can include the rest of your app's specific main process
// code. You can also put them in separate files and import them here.
main()
