import { newE2EPage } from '@stencil/core/testing'

describe('app-onboarding-reporting', () => {
  it.skip('renders', async () => {
    const page = await newE2EPage()
    await page.setContent('<app-onboarding-reporting></app-onboarding-reporting>')

    const element = await page.find('app-onboarding-reporting')
    expect(element).toHaveClass('hydrated')
  })

  it.skip('contains a "Profile Page" button', async () => {
    const page = await newE2EPage()
    await page.setContent('<app-onboarding-reporting></app-onboarding-reporting>')

    const element = await page.find('app-onboarding-reporting >>> button')
    expect(element.textContent).toEqual('Profile page')
  })
})
