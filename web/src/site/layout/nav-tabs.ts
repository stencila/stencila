import { LitElement } from 'lit'
import { customElement } from 'lit/decorators.js'

/**
 * Navigation tabs component for header
 *
 * A Light DOM component that enhances server-rendered navigation tabs with:
 * - Keyboard navigation (arrow keys between tabs)
 *
 * The HTML is server-rendered for SEO.
 */
@customElement('stencila-nav-tabs')
export class StencilaNavTabs extends LitElement {
  private tabs: HTMLAnchorElement[] = []

  /**
   * Use Light DOM so theme CSS applies
   */
  protected override createRenderRoot() {
    return this
  }

  override connectedCallback() {
    super.connectedCallback()

    // Collect all tab links
    this.tabs = Array.from(this.querySelectorAll('a.nav-tab'))

    // Set up keyboard navigation
    this.addEventListener('keydown', this.handleKeydown)
  }

  override disconnectedCallback() {
    super.disconnectedCallback()
    this.removeEventListener('keydown', this.handleKeydown)
  }

  /**
   * Handle keyboard navigation between tabs
   */
  private handleKeydown = (event: KeyboardEvent) => {
    const target = event.target as HTMLElement
    if (!target.matches('a.nav-tab')) return

    const currentIndex = this.tabs.indexOf(target as HTMLAnchorElement)
    if (currentIndex === -1) return

    let nextIndex: number | null = null

    switch (event.key) {
      case 'ArrowLeft':
        nextIndex = currentIndex > 0 ? currentIndex - 1 : this.tabs.length - 1
        break
      case 'ArrowRight':
        nextIndex = currentIndex < this.tabs.length - 1 ? currentIndex + 1 : 0
        break
      case 'Home':
        nextIndex = 0
        break
      case 'End':
        nextIndex = this.tabs.length - 1
        break
    }

    if (nextIndex !== null) {
      event.preventDefault()
      this.tabs[nextIndex].focus()
    }
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'stencila-nav-tabs': StencilaNavTabs
  }
}
