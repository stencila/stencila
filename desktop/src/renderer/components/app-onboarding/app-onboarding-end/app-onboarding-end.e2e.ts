import { newE2EPage } from '@stencil/core/testing'

describe('app-onboarding-end', () => {
  it.skip('renders', async () => {
    const page = await newE2EPage()
    await page.setContent('<app-onboarding-end></app-onboarding-end>')

    const element = await page.find('app-onboarding-end')
    expect(element).toHaveClass('hydrated')
  })

  it.skip('contains a "Profile Page" button', async () => {
    const page = await newE2EPage()
    await page.setContent('<app-onboarding-end></app-onboarding-end>')

    const element = await page.find('app-onboarding-end >>> button')
    expect(element.textContent).toEqual('Profile page')
  })
})
