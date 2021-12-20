import {
  ElectronApplication,
  Page,
  _electron as electron,
} from '@playwright/test'
import type { PageFunctionOn } from 'playwright-core/types/structs'

export const launchApp = async (): Promise<{
  app: ElectronApplication
  page: Page
}> => {
  const app = await electron.launch({
    args: ['.'],
    env: { NODE_ENV: 'production' },
  })

  const page = await app.firstWindow()
  await page.waitForSelector('html.hydrated')

  return {
    app: app,
    page,
  }
}

// Utility functions based on https://github.com/spaceagetv/electron-playwright-example/blob/master/e2e-tests/electron-playwright-helpers.ts
//
// MIT License
//
// Copyright (c) 2021 Jeff Robbins
//
// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

/**
 * Execute the .click() method on the element with the given id.
 * @returns Promise<void> - resolves with the result of the click() method - probably `undefined`
 */
export function clickMenuItemById(
  electronApp: ElectronApplication,
  id: string
): Promise<unknown> {
  return electronApp.evaluate(({ Menu }, menuId) => {
    const menu = Menu.getApplicationMenu()
    const menuItem = menu.getMenuItemById(menuId)
    if (menuItem) {
      return menuItem.click()
    } else {
      throw new Error(`Menu item with id ${menuId} not found`)
    }
  }, id)
}

type MenuItemKey = keyof Electron.MenuItem

export function getMenuItemAttribute(
  electronApp: ElectronApplication,
  menuId: string,
  attribute: MenuItemKey
): Promise<unknown> {
  const resultPromise = electronApp.evaluate(
    ({ Menu }, { menuId, attribute }) => {
      const menu = Menu.getApplicationMenu()
      const menuItem = menu.getMenuItemById(menuId)
      if (menuItem) {
        return menuItem[attribute]
      } else {
        throw new Error(`Menu item with id ${menuId} not found`)
      }
    },
    { menuId, attribute }
  )
  return resultPromise
}

export async function waitForMenuItem(
  electronApp: ElectronApplication,
  id: string
) {
  await electronWaitForFunction(
    electronApp,
    ({ Menu }, id) => {
      const menu = Menu.getApplicationMenu()
      return !!menu.getMenuItemById(id)
    },
    id
  )
}

/**
 * Send an ipcRenderer.send() from a given window.
 * Note: nodeIntegration must be true and contextIsolation must be false
 * in the webPreferences for this window
 * @returns Promise<unknown> - resolves with the result of the function
 */
export function ipcRendererSend(
  window: Page,
  message: string,
  ...args: unknown[]
): Promise<unknown> {
  return window.evaluate(
    ({ message, args }) => {
      // eslint-disable-next-line @typescript-eslint/no-var-requires
      const { ipcRenderer } = require('electron')
      return ipcRenderer.send(message, ...args)
    },
    { message, args }
  )
}

/**
 * Send an ipcRenderer.invoke() from a given window.
 * Note: nodeIntegration must be true and contextIsolation must be false
 * in the webPreferences for this window
 * @returns Promise<unknown> - resolves with the result of the function
 */
export function ipcRendererInvoke(
  window: Page,
  message: string,
  ...args: unknown[]
): Promise<unknown> {
  return window.evaluate(
    async ({ message, args }) => {
      // eslint-disable-next-line @typescript-eslint/no-var-requires
      const { ipcRenderer } = require('electron')
      return await ipcRenderer.invoke(message, ...args)
    },
    { message, args }
  )
}

/**
 * Emit an ipcMain message from the main process.
 * @returns Promise<boolean> - true if there were listeners for this message
 */
export function ipcMainEmit(
  electronApp: ElectronApplication,
  message: string,
  ...args: unknown[]
): Promise<boolean> {
  return electronApp.evaluate(
    ({ ipcMain }, { message, args }) => {
      return ipcMain.emit(message, ...args)
    },
    { message, args }
  )
}

export function ipcMainInvokeFirstListener(
  electronApp: ElectronApplication,
  message: string,
  ...args: unknown[]
): Promise<unknown> {
  return electronApp.evaluate(
    ({ ipcMain }, { message, args }) => {
      if (ipcMain.listenerCount(message) > 0) {
        return ipcMain.listeners(message)[0](...args)
      } else {
        throw new Error(`No listeners for message ${message}`)
      }
    },
    { message, args }
  )
}

/**
 * Wait for a function to evaluate to true in the main Electron process
 * This function is to `electronApp.evaluate()`
 * as `page.waitForFunction()` is `page.evaluate()`
 * @param electronApp
 * @param fn - the function to evaluate in the main process - must return a boolean
 * @param arg - an argument to pass to the function
 */
export async function electronWaitForFunction<R, Arg>(
  electronApp: ElectronApplication,
  fn: PageFunctionOn<typeof Electron.CrossProcessExports, Arg, R>,
  arg?: Arg
): Promise<void> {
  while (!(await electronApp.evaluate(fn, arg))) {
    // wait 100ms before trying again
    await new Promise((resolve) => setTimeout(resolve, 100))
  }
}
