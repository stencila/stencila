import { argosScreenshot } from '@argos-ci/playwright'
import { test, expect } from '@playwright/test'

const url = 'article-ark/article-ark'

test('has title', async ({ page }) => {
  await page.goto(url)
  await expect(page).toHaveTitle('Stencila')
})

test('show document menu', async ({ page }) => {
  await page.goto(url)

  const menuElement = await page.locator('stencila-document-menu')

  const dropdown = await menuElement.locator('sl-menu')

  await expect(dropdown).toBeHidden()

  const trigger = menuElement.locator('div[slot="trigger"]')

  await trigger.hover()
  await expect(dropdown).toBeVisible()
})

test('argos screenshot article ark', async ({ page }) => {
  await page.goto(url)
  await argosScreenshot(page, 'article-ark')
})
