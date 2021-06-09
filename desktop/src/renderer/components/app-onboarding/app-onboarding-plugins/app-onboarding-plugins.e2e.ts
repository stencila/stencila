import { newE2EPage } from '@stencil/core/testing'

describe('app-onboarding-plugins', () => {
  it.skip('renders', async () => {
    const page = await newE2EPage()
    await page.setContent('<app-onboarding-plugins></app-onboarding-plugins>')

    const element = await page.find('app-onboarding-plugins')
    expect(element).toHaveClass('hydrated')
  })

  it.skip('contains a "Profile Page" button', async () => {
    const page = await newE2EPage()
    await page.setContent('<app-onboarding-plugins></app-onboarding-plugins>')

    const element = await page.find('app-onboarding-plugins >>> button')
    expect(element.textContent).toEqual('Profile page')
  })
})
