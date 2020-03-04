import { append, create } from '../../util'

export const forceReady = (doc?: Document | null): void => {
  if (doc === null || doc === undefined) return

  doc.dispatchEvent(
    new Event('DOMContentLoaded', {
      bubbles: true,
      cancelable: true
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
    }
  }
}
