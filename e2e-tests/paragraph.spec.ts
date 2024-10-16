import { test, expect } from '@playwright/test';

test('has `stencila-paragraph` elements', async ({ page }) => {
  await page.goto('/examples/tests/paragraph/paragraph')

  const paragraphs = await page.locator('stencila-paragraph').all()
  expect(paragraphs.length).toBe(4)
})
