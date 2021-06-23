import { newE2EPage } from '@stencil/core/testing'

const electronMock = {
  invoke: () => Promise.resolve('results'),
  receive: () => Promise.resolve(),
  send: () => Promise.resolve(),
  remove: () => Promise.resolve(),
  removeAll: () => Promise.resolve(),
}

describe('app-document-editor', () => {
  it.skip('renders', async () => {
    const page = await newE2EPage({ failOnConsoleError: true })

    window.api = electronMock
    console.log(window)

    await page.setContent(
      '<app-document-editor documentId="test"></app-document-editor>'
    )

    const element = await page.find('app-document-editor')
    expect(element).toHaveClass('hydrated')
  })

  it.skip('displays the specified name', async () => {
    const page = await newE2EPage({ url: '/project/joseph' })

    const projectElement = await page.find('app-root >>> app-document-editor')
    const element = projectElement.shadowRoot.querySelector('div')
    expect(element?.textContent).toContain('Hello! My name is Joseph.')
  })
})
