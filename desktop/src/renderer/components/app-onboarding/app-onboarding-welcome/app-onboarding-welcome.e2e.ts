import { newE2EPage } from '@stencil/core/testing'

describe('app-onboarding-welcome', () => {
  it.skip('renders', async () => {
    const page = await newE2EPage()
    await page.setContent('<app-onboarding-welcome></app-onboarding-welcome>')

    const element = await page.find('app-onboarding-welcome')
    expect(element).toHaveClass('hydrated')
  })

  it.skip('contains a "Profile Page" button', async () => {
    const page = await newE2EPage()
    await page.setContent('<app-onboarding-welcome></app-onboarding-welcome>')

    const element = await page.find('app-onboarding-welcome >>> button')
    expect(element.textContent).toEqual('Profile page')
  })
})
