import { BrowserWindow } from 'electron'
import { scheme } from '../app-protocol'

const isTrustedOrigin = (url: string): boolean =>
  url.startsWith('http://localhost') || url.startsWith(`${scheme}://rse`)

// Enable security best practices
// @see: https://github.com/doyensec/electronegativity/wiki
export const hardenWindow = (win: BrowserWindow) => {
  // @see: https://github.com/doyensec/electronegativity/wiki/LIMIT_NAVIGATION_JS_CHECK
  win.webContents.on('new-window', (e, url) => {
    if (!isTrustedOrigin(url)) {
      e.preventDefault()
    }
  })

  win.webContents.on('will-navigate', (e, url) => {
    if (!isTrustedOrigin(url)) {
      e.preventDefault()
    }
  })

  win.webContents.setWindowOpenHandler(({ url }) => {
    if (!isTrustedOrigin(url)) {
      return { action: 'deny' }
    }

    return {
      action: 'allow',
    }
  })

  // https://github.com/doyensec/electronegativity/wiki/PERMISSION_REQUEST_HANDLER_JS_CHECK
  win.webContents.session.setPermissionRequestHandler(
    (webContents, permission, callback) => {
      if (
        !isTrustedOrigin(webContents.getURL()) &&
        permission === 'openExternal'
      ) {
        return callback(false)
      } else {
        return callback(true)
      }
    }
  )
}
