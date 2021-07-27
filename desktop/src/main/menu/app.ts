import { app, MenuItemConstructorOptions } from 'electron'
import { showSettings } from '../config/window'
import { checkForUpdates } from '../utils/update'
import { isMac } from './utils'

export const baseAppMenu: MenuItemConstructorOptions = isMac
  ? {
      label: app.name,
      submenu: [
        { role: 'about' as const },
        { label: 'Check For Updates…', click: checkForUpdates },
        { type: 'separator' as const },
        {
          label: 'Preferences…',
          accelerator: 'CommandOrControl+,',
          click: () => {
            showSettings()
          },
        },
        { type: 'separator' as const },
        { role: 'services' as const },
        { type: 'separator' as const },
        { role: 'hide' as const },
        { role: 'hideOthers' as const },
        { role: 'unhide' as const },
        { type: 'separator' as const },
        { role: 'quit' as const },
      ],
    }
  : {}
