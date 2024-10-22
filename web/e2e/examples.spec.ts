import * as path from 'path'
import { fileURLToPath } from 'url'

import { argosScreenshot } from '@argos-ci/playwright'
import { test } from '@playwright/test'
import { globSync } from 'glob'

const testExpandAllStyleTag = `
  :root { 
    --lastmod-display: none; 
  }
`

const dirname = path.dirname(fileURLToPath(import.meta.url))
const examples = path.join(dirname, '..', '..', 'examples', 'web')
const files = globSync('**/*.smd', { cwd: examples })

const modes = ['', 'test-expand-all']

for (const file of files) {
  const fileName = file.replace(/\.smd$/, '').replace(/\\/g, '/')

  for (const mode of modes) {
    const testName =
      mode === '' ? fileName : `${fileName}-${mode.replace('test-', '')}`
    const url = mode === '' ? fileName : `${fileName}?mode=${mode}`

    test(testName, async ({ page }) => {
      await page.goto(url)
      if (mode === 'test-expand-all') {
        await page.addStyleTag({
          content: testExpandAllStyleTag,
        })
      }
      await argosScreenshot(page, testName)
    })
  }
}
