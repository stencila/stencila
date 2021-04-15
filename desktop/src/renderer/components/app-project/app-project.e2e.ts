import { newE2EPage } from '@stencil/core/testing'

describe('app-project', () => {
  it('renders', async () => {
    const page = await newE2EPage()
    await page.setContent('<app-project></app-project>')

    const element = await page.find('app-project')
    expect(element).toHaveClass('hydrated')
  })

  it('displays the specified name', async () => {
    const page = await newE2EPage({ url: '/project/joseph' })

    const projectElement = await page.find('app-root >>> app-project')
    const element = projectElement.shadowRoot.querySelector('div')
    expect(element?.textContent).toContain('Hello! My name is Joseph.')
  })

  // it('includes a div with the class "app-project"', async () => {
  //   const page = await newE2EPage({ url: '/project/joseph' });

  // I would like to use a selector like this above, but it does not seem to work
  //   const element = await page.find('app-root >>> app-project >>> div');
  //   expect(element).toHaveClass('app-project');
  // });
})
