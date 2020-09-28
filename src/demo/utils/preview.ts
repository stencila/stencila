import { keys, ASSET_PATH } from '.'
import { examples, resolveExample } from '../../examples'
import { append, create } from '../../util'

/**
 * Read query parameters from the URL, and conditionally hide the Theme Editor UI components
 */
export const initUiVisibility = (): void => {
  const query = window.location.search
  const hideUi: boolean = query.includes('ui=false')

  if (hideUi || query.includes('header=false')) {
    document.body.classList.add('headerHidden')
  }

  if (hideUi || query.includes('sidebar=false')) {
    document.body.classList.add('sideBarHidden')
  }
}

export const getExample = (): string => {
  return (
    new URL(window.location.href).searchParams.get(keys.EXAMPLE) ??
    sessionStorage.getItem(keys.EXAMPLE) ??
    examples.articleKitchenSink
  )
}

export const setExample = (example: string): void => {
  const url = new URL(window.location.href)
  sessionStorage.setItem(keys.EXAMPLE, example)

  if (url.searchParams.get(keys.EXAMPLE) !== example) {
    url.searchParams.set(keys.EXAMPLE, example)
    history.replaceState(null, 'none', url.toString())
  }

  const preview = getPreview()
  if (preview !== null && !preview.getAttribute('src')?.includes(example)) {
    preview.setAttribute(
      'src',
      `${ASSET_PATH}examples/${resolveExample(example)}.html`
    )
  }
}

export const forceReady = (doc?: Document | null): void => {
  if (doc === null || doc === undefined) return

  doc.dispatchEvent(
    new Event('DOMContentLoaded', {
      bubbles: true,
      cancelable: true,
    })
  )
}

export const getPreview = (): HTMLIFrameElement | null =>
  document.getElementsByTagName('iframe')[0] ?? null

export const getPreviewDoc = (): Document | null => {
  const preview: HTMLIFrameElement | null =
    document.getElementsByTagName('iframe')[0] ?? null
  return preview !== null ? preview.contentDocument : null
}

export const getPreviewHead = (): HTMLHeadElement | null => {
  return getPreviewDoc()?.getElementsByTagName('head')[0] ?? null
}

const injectExecutableToolbar = () => {
  const query = window.location.search
  const toolbarDisabled: boolean = query.includes('toolbar=false')

  if (toolbarDisabled) {
    return
  }

  const doc = getPreviewDoc()
  const article = doc?.querySelector('[data-itemscope="root"]')

  if (doc && article) {
    const toolbar = doc.createElement('stencila-executable-document-toolbar')
    toolbar.setAttribute('source-url', '#')

    article.prepend(toolbar)
  }
}

/**
 * Inject necessary stylesheets and scripts to fully render a document
 * Currently this function inject the scripts for Stencila Components. Note that you will need to trigger the
 * `DOMContentLoaded` event manually by callign `forceReady()` after invoking this funcion.
 *
 * @function injectPreviewAssets
 */
export const injectPreviewAssets = (): void => {
  const previewHead = getPreviewHead()

  if (previewHead != null) {
    const stencilaComponentsES6 = create('script')
    stencilaComponentsES6.setAttribute('type', 'module')
    stencilaComponentsES6.setAttribute(
      'src',
      `https://unpkg.com/@stencila/components@latest/dist/stencila-components/stencila-components.esm.js`
    )

    const stencilaComponents = create('script')
    stencilaComponents.setAttribute('type', 'text/javascript')
    stencilaComponents.setAttribute('nomodule', 'true')
    stencilaComponents.setAttribute(
      'src',
      `https://unpkg.com/@stencila/components@latest/dist/stencila-components/stencila-components.js`
    )

    append(previewHead, stencilaComponentsES6)
    append(previewHead, stencilaComponents)

    injectExecutableToolbar()
  }
}
