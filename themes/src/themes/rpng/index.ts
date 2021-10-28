import '../../extensions/cite-apa'
import { ready } from '../../util'

ready(async () => {
  /**
   * Wait until CodeChunk and CodeExpression Web Components have been loaded, and trigger the custom CodeChunk event
   * to collapse the source code panel
   */
  await Promise.all([
    customElements.whenDefined('stencila-code-chunk'),
    customElements.whenDefined('stencila-code-expression'),
  ])

  window.dispatchEvent(
    new CustomEvent('collapseAllCode', {
      bubbles: true,
      cancelable: true,
      detail: { isVisible: false },
    })
  )
})
