import { newE2EPage } from '@stencil/core/testing'

describe('project-root', () => {
  it.skip('renders', async () => {
    const page = await newE2EPage()
    await page.setContent('<project-root></project-root>')

    const element = await page.find('project-root')
    expect(element).toHaveClass('hydrated')
  })

  it.skip('contains a "Profile Page" button', async () => {
    const page = await newE2EPage()
    await page.setContent('<project-root></project-root>')

    const element = await page.find('project-root >>> button')
    expect(element.textContent).toEqual('Profile page')
  })
})
