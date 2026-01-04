import { LitElement } from 'lit'
import { customElement } from 'lit/decorators.js'

import { GLIDE_REQUEST } from '../navigation/events'

/**
 * Page navigation component (prev/next links)
 *
 * A Light DOM component that displays prev/next page navigation links.
 * The HTML is server-rendered for SEO and accessibility.
 *
 * Features:
 * - Keyboard navigation (j/k for prev/next)
 * - Semantic HTML with rel="prev" and rel="next"
 * - CSS-based styling with arrow indicators
 */
@customElement('stencila-page-nav')
export class StencilaPageNav extends LitElement {
  /**
   * Use Light DOM so theme CSS applies
   */
  protected override createRenderRoot() {
    return this
  }

  override connectedCallback() {
    super.connectedCallback()

    // Add keyboard navigation support
    this.setupKeyboardNavigation()
  }

  override disconnectedCallback() {
    super.disconnectedCallback()

    // Remove keyboard listener
    document.removeEventListener('keydown', this.handleKeydown)
  }

  /**
   * Set up keyboard navigation
   *
   * j - Go to previous page
   * k - Go to next page
   */
  private setupKeyboardNavigation() {
    document.addEventListener('keydown', this.handleKeydown)
  }

  private handleKeydown = (event: KeyboardEvent) => {
    // Ignore if user is typing in an input
    const target = event.target as HTMLElement
    if (
      target.tagName === 'INPUT' ||
      target.tagName === 'TEXTAREA' ||
      target.isContentEditable
    ) {
      return
    }

    // Ignore if modifier keys are pressed
    if (event.ctrlKey || event.metaKey || event.altKey) {
      return
    }

    switch (event.key) {
      case 'j':
      case 'ArrowLeft':
        this.navigatePrev()
        break
      case 'k':
      case 'ArrowRight':
        this.navigateNext()
        break
    }
  }

  private navigatePrev() {
    const prevLink = this.querySelector<HTMLAnchorElement>('.page-nav-prev')
    if (prevLink?.href) {
      // Use client-side navigation if available
      window.dispatchEvent(
        new CustomEvent(GLIDE_REQUEST, {
          detail: { url: prevLink.href, trigger: 'keyboard' },
        })
      )
    }
  }

  private navigateNext() {
    const nextLink = this.querySelector<HTMLAnchorElement>('.page-nav-next')
    if (nextLink?.href) {
      // Use client-side navigation if available
      window.dispatchEvent(
        new CustomEvent(GLIDE_REQUEST, {
          detail: { url: nextLink.href, trigger: 'keyboard' },
        })
      )
    }
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'stencila-page-nav': StencilaPageNav
  }
}
