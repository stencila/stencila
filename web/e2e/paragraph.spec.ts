import { test, expect } from '@playwright/test'

test('has `stencila-paragraph` elements', async ({ page }) => {
  await page.goto('paragraph/paragraph')

  const paragraphs = await page.locator('stencila-paragraph').all()
  expect(paragraphs.length).toBe(4)
})
