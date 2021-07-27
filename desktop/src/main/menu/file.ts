import { MenuItemConstructorOptions } from 'electron'
import { showSettings } from '../config/window'
import { openProject } from '../project/handlers'
import { isWindows } from './utils'

export const baseFileMenu: MenuItemConstructorOptions = {
  label: 'File',
  submenu: [
    // { label: 'New' },
    // { type: 'separator' },
    {
      label: 'Open…',
      accelerator: 'CommandOrControl+o',
      click: openProject,
    },
    { type: 'separator' },
    {
      role: 'close' as const,
      accelerator: isWindows ? 'Alt+F4' : 'CommandOrControl+w',
    },
    { type: 'separator' as const },
    {
      label: 'Preferences…',
      accelerator: 'CommandOrControl+,',
      click: () => {
        showSettings()
      },
    },
    { type: 'separator' as const },
    { role: 'quit' as const },
  ],
}
