import { LitElement } from 'lit'
import { customElement } from 'lit/decorators.js'

import { GlideEvents } from '../glide/events'

/**
 * Navigation groups component
 *
 * Provides SPA navigation support for footer-style grouped links.
 * Updates active state when navigating between pages.
 *
 * The HTML structure is rendered server-side by Rust. This component
 * adds client-side interactivity for SPA navigation.
 */
@customElement('stencila-nav-groups')
export class StencilaNavGroups extends LitElement {
  /**
   * Use Light DOM so theme CSS can style the component
   */
  protected override createRenderRoot() {
    return this
  }

  override connectedCallback() {
    super.connectedCallback()

    // Listen for SPA navigation to update active state
    window.addEventListener(GlideEvents.END, this.handleGlideEnd)
  }

  override disconnectedCallback() {
    super.disconnectedCallback()
    window.removeEventListener(GlideEvents.END, this.handleGlideEnd)
  }

  /**
   * Handle SPA navigation end event
   */
  private handleGlideEnd = () => {
    // Small delay to ensure DOM is updated
    requestAnimationFrame(() => {
      this.updateActiveState()
    })
  }

  /**
   * Update the active state based on current URL
   */
  private updateActiveState() {
    const currentPath = window.location.pathname

    // Remove active state from all link items
    const activeLinks = this.querySelectorAll('.link[data-active="true"]')
    for (const link of activeLinks) {
      link.setAttribute('data-active', 'false')
      const anchor = link.querySelector('a')
      if (anchor) {
        anchor.removeAttribute('aria-current')
      }
    }

    // Remove aria-current from all group heading links
    const headingLinks = this.querySelectorAll<HTMLAnchorElement>(
      '.group-heading a[aria-current]'
    )
    for (const anchor of headingLinks) {
      anchor.removeAttribute('aria-current')
    }

    // Find and activate matching links (both .link items and group headings)
    const allAnchors = this.querySelectorAll<HTMLAnchorElement>(
      '.link a[href], .group-heading a[href]'
    )
    for (const anchor of allAnchors) {
      const href = anchor.getAttribute('href')
      // Match exact path or path with trailing slash
      if (href === currentPath || href === currentPath + '/') {
        anchor.setAttribute('aria-current', 'page')

        // Also set data-active on .link items
        const linkItem = anchor.closest('.link')
        if (linkItem) {
          linkItem.setAttribute('data-active', 'true')
        }
      }
    }
  }
}
