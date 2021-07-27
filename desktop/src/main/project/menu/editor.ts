import { MenuItemConstructorOptions } from 'electron'
import {
  isLineNumbersEnabled,
  isLineWrappingEnabled,
  toggleLineNumbers,
  toggleLineWrapping,
} from '../../store/handlers'

export const projectEditorMenu: MenuItemConstructorOptions = {
  label: 'Editor',
  submenu: [
    {
      label: 'Wrap Lines',
      click: () => {
        toggleLineWrapping()
      },
      checked: isLineWrappingEnabled(),
      type: 'checkbox',
    },
    {
      label: 'Line Numbers',
      click: () => {
        toggleLineNumbers()
      },
      checked: isLineNumbersEnabled(),
      type: 'checkbox',
    },
  ],
}
