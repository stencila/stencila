import { Menu, MenuItem, MenuItemConstructorOptions } from 'electron'
import { baseAppMenu } from '../../menu/app'
import { baseHelpMenu } from '../../menu/help'
import { baseViewMenu } from '../../menu/view'
import { baseWindowMenu } from '../../menu/window'
import { projectEditMenu } from './edit'
import { projectEditorMenu } from './editor'
import { projectFileMenu } from './file'

const template: (MenuItemConstructorOptions | MenuItem)[] = [
  baseAppMenu,
  projectFileMenu,
  projectEditMenu,
  baseViewMenu,
  projectEditorMenu,
  baseWindowMenu,
  baseHelpMenu,
]

const menu = Menu.buildFromTemplate(template)
export const registerProjectMenu = () => Menu.setApplicationMenu(menu)
