import { newE2EPage } from '@stencil/core/testing'

describe('app-project-file-preview', () => {
  it('renders', async () => {
    const page = await newE2EPage()
    await page.setContent('<app-project-file-preview></app-project-file-preview>')

    const element = await page.find('app-project-file-preview')
    expect(element).toHaveClass('hydrated')
  })

  it.skip('displays the specified name', async () => {
    const page = await newE2EPage({ url: '/project/joseph' })

    const projectElement = await page.find('app-root >>> app-project-file-preview')
    const element = projectElement.shadowRoot.querySelector('div')
    expect(element?.textContent).toContain('Hello! My name is Joseph.')
  })
})
