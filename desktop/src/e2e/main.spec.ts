import { ElectronApplication, expect, Page, test } from '@playwright/test'
import { version } from '../../package.json'
import { launchApp } from './helpers'
import path from 'path'

test.describe('Launcher', () => {
  let electronApp: ElectronApplication
  let appWindow: Page

  test.beforeAll(async () => {
    const { app, page } = await launchApp()
    electronApp = app
    appWindow = page
  })

  test.afterAll(async () => {
    await electronApp.close()
  })

  test('Launcher displays version number matching one in package.json', async () => {
    const appVersion = appWindow.locator('.appVersion')
    await expect(appVersion).toContainText(version)
  })

  test.skip('Open project', async () => {
    // TODO: Revisit once `filechooser` dialog is supported by Playwright
    // @see https://github.com/microsoft/playwright/issues/8278
    const fixturesPath = path.join(__dirname, '..', '..', '..', 'fixtures')

    appWindow.on('filechooser', async (fileChooser) => {
      await fileChooser.setFiles(fixturesPath)
    })

    await appWindow.click('stencila-button:has-text("Open folder…")')
    // const allWindows = await electronApp.windows()
  })
})
