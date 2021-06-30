// Use GitHub releases to detect new versions prompt user to update the app.
// @see https://www.electronforge.io/advanced/auto-update
// @see https://github.com/electron/update-electron-app
import autoUpdate from 'update-electron-app'
import { isProduction } from '../../preload/utils/env'

export const checkForUpdates = () => {
  if (isProduction) {
    autoUpdate({ updateInterval: '8 hours' })
  }
}
