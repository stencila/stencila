import { test, expect } from '@playwright/test'

const url = 'paragraph/simple'

test('show document menu', async ({ page }) => {
  await page.goto(url)

  const menuElement = await page.locator('stencila-document-menu')

  const dropdown = await menuElement.locator('sl-menu')

  await expect(dropdown).toBeHidden()

  const trigger = menuElement.locator('div[slot="trigger"]')

  await trigger.hover()
  await expect(dropdown).toBeVisible()
})
