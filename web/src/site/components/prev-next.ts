import { LitElement } from 'lit'
import { customElement } from 'lit/decorators.js'

import { GLIDE_REQUEST } from '../glide/events'

/**
 * Previous/next navigation component
 *
 * A component that provides navigation links to previous and next pages.
 * Content is rendered server-side; this component adds keyboard navigation.
 *
 * Keyboard shortcuts:
 * - j or ArrowLeft: Navigate to previous page
 * - k or ArrowRight: Navigate to next page
 */
@customElement('stencila-prev-next')
export class StencilaPrevNext extends LitElement {
  /**
   * Use Light DOM so theme CSS can style the component
   */
  protected override createRenderRoot() {
    return this
  }

  override connectedCallback() {
    super.connectedCallback()
    document.addEventListener('keydown', this.handleKeydown)
  }

  override disconnectedCallback() {
    super.disconnectedCallback()
    document.removeEventListener('keydown', this.handleKeydown)
  }

  /**
   * Handle keyboard navigation
   */
  private handleKeydown = (event: KeyboardEvent) => {
    // Skip if typing in input/textarea/contenteditable
    const target = event.target as HTMLElement
    if (
      target.tagName === 'INPUT' ||
      target.tagName === 'TEXTAREA' ||
      target.isContentEditable
    ) {
      return
    }

    // Skip if modifier keys pressed (allow browser shortcuts)
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

  /**
   * Navigate to the previous page
   */
  private navigatePrev() {
    const link = this.querySelector<HTMLAnchorElement>('.prev')
    if (link?.href) {
      window.dispatchEvent(
        new CustomEvent(GLIDE_REQUEST, {
          detail: { url: link.href, trigger: 'keyboard' },
        })
      )
    }
  }

  /**
   * Navigate to the next page
   */
  private navigateNext() {
    const link = this.querySelector<HTMLAnchorElement>('.next')
    if (link?.href) {
      window.dispatchEvent(
        new CustomEvent(GLIDE_REQUEST, {
          detail: { url: link.href, trigger: 'keyboard' },
        })
      )
    }
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'stencila-prev-next': StencilaPrevNext
  }
}
