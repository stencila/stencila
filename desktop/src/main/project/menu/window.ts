import { MenuItemConstructorOptions } from 'electron'
import { baseWindowSubMenu } from '../../menu/window'
import { cycleToNextTab, cycleToPreviousTab } from '../utils'

export const projectWindowMenu: MenuItemConstructorOptions = {
  label: 'Window',
  submenu: [
    {
      label: 'Previous Tab',
      accelerator: 'CommandOrControl+Shift+[',
      click: () => {
        cycleToPreviousTab()
      },
    },
    {
      label: 'Next Tab',
      accelerator: 'CommandOrControl+Shift+]',
      click: () => {
        cycleToNextTab()
      },
    },
    { type: 'separator' },
    ...(baseWindowSubMenu ?? []),
  ],
}
