import { test, expect } from '@playwright/test';

test('has `stencila-heading` elements', async ({ page }) => {
  await page.goto('/examples/tests/heading/heading');
  
  const headings = await page.locator('stencila-heading').all()
  expect(headings.length).toBe(7)
})
