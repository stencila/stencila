import { newE2EPage } from '@stencil/core/testing'

describe('app-settings-root', () => {
  it('renders', async () => {
    const page = await newE2EPage()
    await page.setContent('<app-settings-root></app-settings-root>')

    const element = await page.find('app-settings-root')
    expect(element).toHaveClass('hydrated')
  })

  it('contains sidebar navigation', async () => {
    const page = await newE2EPage()
    await page.setContent('<app-settings-root></app-settings-root>')

    const element = await page.find('app-settings-root app-settings-sidebar')
    expect(element).toBeDefined()
  })
})
