import { newE2EPage } from '@stencil/core/testing'

describe('app-project-sidebar-files', () => {
  it('renders', async () => {
    const page = await newE2EPage()
    await page.setContent('<app-project-sidebar-files></app-project-sidebar-files>')

    const element = await page.find('app-project-sidebar-files')
    expect(element).toHaveClass('hydrated')
  })

  it.skip('displays the specified name', async () => {
    const page = await newE2EPage({ url: '/project/joseph' })

    const projectElement = await page.find('app-root >>> app-project-sidebar-files')
    const element = projectElement.shadowRoot.querySelector('div')
    expect(element?.textContent).toContain('Hello! My name is Joseph.')
  })
})
