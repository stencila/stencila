import { newE2EPage } from '@stencil/core/testing'

describe('project-sidebar-files', () => {
  it('renders', async () => {
    const page = await newE2EPage()
    await page.setContent('<project-sidebar-files></project-sidebar-files>')

    const element = await page.find('project-sidebar-files')
    expect(element).toHaveClass('hydrated')
  })

  it.skip('displays the specified name', async () => {
    const page = await newE2EPage({ url: '/project/joseph' })

    const projectElement = await page.find('app-root >>> project-sidebar-files')
    const element = projectElement.shadowRoot.querySelector('div')
    expect(element?.textContent).toContain('Hello! My name is Joseph.')
  })
})
