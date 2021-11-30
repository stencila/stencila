import { MenuItemConstructorOptions } from 'electron'
import { showSettings } from '../../config/window'
import { isWindows } from '../../menu/utils'
import {
  closeActiveTab,
  createNewDocument,
  saveActiveDoc,
  saveActiveDocAs,
} from '../utils'
import { openProject } from '../handlers'

export const projectFileMenu: MenuItemConstructorOptions = {
  label: 'File',
  submenu: [
    {
      label: 'New File',
      accelerator: 'CommandOrControl+N',
      click: (): void => {
        createNewDocument()
      },
    },
    { type: 'separator' },
    {
      label: 'Open…',
      accelerator: 'CommandOrControl+o',
      click: () => {
        openProject().catch((err) => {
          console.log('Could not open project\n', err)
        })
      },
    },
    { type: 'separator' },
    {
      label: 'Save',
      click: (): void => {
        saveActiveDoc()
      },
      accelerator: 'CommandOrControl+s',
    },
    {
      label: 'Save as…',
      click: (): void => {
        saveActiveDocAs()
      },
      accelerator: 'CommandOrControl+Shift+s',
    },
    { type: 'separator' },
    {
      label: 'Close Tab' as const,
      click: (): void => {
        closeActiveTab().catch((err) => {
          console.log('Could not close tab\n', err)
        })
      },
      accelerator: isWindows ? 'Control+Shift+W' : 'CommandOrControl+w',
    },
    {
      role: 'close' as const,
      accelerator: isWindows ? 'Alt+F4' : 'CommandOrControl+Shift+w',
    },
    { type: 'separator' as const },
    {
      label: 'Preferences…',
      accelerator: 'CommandOrControl+,',
      click: (): void => {
        showSettings()
      },
    },
    { type: 'separator' as const },
    { role: 'quit' as const },
  ],
}
