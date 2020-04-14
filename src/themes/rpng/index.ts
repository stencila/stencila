import { ready } from '../../util'

/**
 * Check whether the custom Web Components have been loaded or not
 */
const documentHydrated = (): boolean =>
  document.querySelector('html')?.classList.contains('hydrated') ?? false

ready(() => {
  /**
   * Wait until Web Components have been loaded, and trigger the custom CodeChunk event
   * to collapse the source code panel
   */
  const poll: number = window.setInterval(() => {
    if (documentHydrated()) {
      window.clearInterval(poll)
      window.dispatchEvent(
        new CustomEvent('collapseAllCode', {
          bubbles: true,
          cancelable: true,
          detail: { isCollapsed: true },
        })
      )
    }
  }, 200)
})
