import { newE2EPage } from '@stencil/core/testing'

describe('app-document-preview', () => {
  it('renders', async () => {
    const page = await newE2EPage()
    await page.setContent('<app-document-preview></app-document-preview>')

    const element = await page.find('app-document-preview')
    expect(element).toHaveClass('hydrated')
  })

  it.skip('displays the specified name', async () => {
    const page = await newE2EPage({ url: '/project/joseph' })

    const projectElement = await page.find('app-root >>> app-document-preview')
    const element = projectElement.shadowRoot.querySelector('div')
    expect(element?.textContent).toContain('Hello! My name is Joseph.')
  })
})
