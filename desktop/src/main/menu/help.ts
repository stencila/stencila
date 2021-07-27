import { MenuItemConstructorOptions } from 'electron'
import { openOnboardingWindow } from '../onboarding/window'
import { checkForUpdates } from '../utils/update'
import { isMac } from './utils'

export const baseHelpMenu: MenuItemConstructorOptions = {
  role: 'help',
  submenu: [
    {
      label: isMac ? 'Stencila Help' : 'Help Center',
      click: async () => {
        const { shell } = require('electron')
        await shell.openExternal('http://help.stenci.la')
      },
    },
    { type: 'separator' },
    {
      label: 'Report an Issue…',
      click: async () => {
        const { shell } = require('electron')
        await shell.openExternal(
          'https://github.com/stencila/stencila/issues/new'
        )
      },
    },
    {
      label: 'Request a Feature…',
      click: async () => {
        const { shell } = require('electron')
        await shell.openExternal(
          'https://github.com/stencila/stencila/discussions/new'
        )
      },
    },
    { type: 'separator' },
    {
      label: 'Check for Updates…',
      click: () => {
        checkForUpdates()
      },
    },
    {
      label: 'Setup…',
      click: openOnboardingWindow,
    },
    { type: 'separator' },
    {
      label: 'Stencila Hub',
      click: async () => {
        const { shell } = require('electron')
        await shell.openExternal('https://hub.stenci.la')
      },
    },
    {
      label: 'Stencila Homepage',
      click: async () => {
        const { shell } = require('electron')
        await shell.openExternal('https://stenci.la')
      },
    },
  ],
}
