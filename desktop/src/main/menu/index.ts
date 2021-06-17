import { app, Menu, MenuItem, MenuItemConstructorOptions } from 'electron'
import { showSettings } from '../config/window'
import { openLauncherWindow } from '../launcher/window'
import { openOnboardingWindow } from '../onboarding/window'
import { openProject } from '../project/handlers'
import { closeActiveTab, saveActiveDoc } from '../window/windowUtils'

const isMac = process.platform === 'darwin'
const isWindows = process.platform === 'win32'

const template: (MenuItemConstructorOptions | MenuItem)[] = [
  // { role: 'appMenu' }
  ...(isMac
    ? [
        {
          label: app.name,
          submenu: [
            { role: 'about' as const },
            { type: 'separator' as const },
            {
              label: 'Preferences…',
              accelerator: 'CommandOrControl+,',
              click: showSettings,
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
        },
      ]
    : []),
  // { role: 'fileMenu' }
  {
    label: 'File',
    submenu: [
      // { label: 'New' },
      // { type: 'separator' },
      { label: 'Open…', click: openProject, accelerator: 'CommandOrControl+o' },
      // { label: 'Open Recent' },
      { type: 'separator' },
      {
        label: 'Save',
        click: saveActiveDoc,
        accelerator: 'CommandOrControl+s',
      },
      { type: 'separator' },

      {
        label: 'Close Tab' as const,
        click: closeActiveTab,
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
        click: showSettings,
      },
      { type: 'separator' as const },
      { role: 'quit' as const },
    ],
  },
  // { role: 'editMenu' }
  {
    label: 'Edit',
    submenu: [
      { role: 'undo' },
      { role: 'redo' },
      { type: 'separator' },
      { role: 'cut' },
      { role: 'copy' },
      { role: 'paste' },
      ...(isMac
        ? [
            { role: 'pasteAndMatchStyle' as const },
            { role: 'delete' as const },
            { role: 'selectAll' as const },
            { type: 'separator' as const },
            {
              label: 'Speech',
              submenu: [
                { role: 'startSpeaking' as const },
                { role: 'stopSpeaking' as const },
              ],
            },
          ]
        : [
            { role: 'delete' as const },
            { type: 'separator' as const },
            { role: 'selectAll' as const },
          ]),
    ],
  },
  // { role: 'viewMenu' }
  {
    label: 'View',
    submenu: [
      { role: 'reload' },
      { role: 'forceReload' },
      { role: 'toggleDevTools' },
      { type: 'separator' },
      { role: 'resetZoom' },
      { role: 'zoomIn' },
      { role: 'zoomOut' },
      { type: 'separator' },
      { role: 'togglefullscreen' },
    ],
  },
  // { role: 'windowMenu' }
  {
    label: 'Window',
    submenu: [
      { role: 'minimize' },
      { role: 'zoom' },
      { type: 'separator' },
      {
        label: 'Launcher',
        accelerator: 'Shift+CommandOrControl+1',
        click: openLauncherWindow,
      },
      ...(isMac
        ? [
            { type: 'separator' as const },
            { role: 'front' as const },
            { type: 'separator' as const },
            { role: 'window' as const },
          ]
        : [{ role: 'close' as const }]),
    ],
  },
  {
    role: 'help',
    submenu: [
      {
        label: 'Stencila Help',
        click: async () => {
          const { shell } = require('electron')
          await shell.openExternal('http://help.stenci.la')
        },
      },
      {
        label: 'Learn More',
        click: async () => {
          const { shell } = require('electron')
          await shell.openExternal('https://stenci.la')
        },
      },
      { type: 'separator' },
      {
        label: 'Report a Problem or Feature Request…',
        click: async () => {
          const { shell } = require('electron')
          await shell.openExternal(
            'https://github.com/stencila/stencila/issues/new'
          )
        },
      },
      { type: 'separator' },
      {
        label: 'Open initial setup screen',
        click: openOnboardingWindow,
      },
    ],
  },
]

const menu = Menu.buildFromTemplate(template)
export const registerMenu = () => Menu.setApplicationMenu(menu)
