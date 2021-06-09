import { newE2EPage } from '@stencil/core/testing'

describe('app-onboarding-root', () => {
  it.skip('renders', async () => {
    const page = await newE2EPage()
    await page.setContent('<app-onboarding-root></app-onboarding-root>')

    const element = await page.find('app-onboarding-root')
    expect(element).toHaveClass('hydrated')
  })

  it.skip('contains a "Profile Page" button', async () => {
    const page = await newE2EPage()
    await page.setContent('<app-onboarding-root></app-onboarding-root>')

    const element = await page.find('app-onboarding-root >>> button')
    expect(element.textContent).toEqual('Profile page')
  })
})
