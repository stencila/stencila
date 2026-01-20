import { LitElement } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { GlideEvents } from '../glide/events'

/**
 * Navigation tree component
 *
 * Provides interactivity for site navigation:
 * - Collapsible groups with toggle buttons
 * - Active page highlighting with SPA navigation support
 * - Keyboard navigation (arrow keys)
 * - Auto-scroll to show active item
 *
 * The HTML structure is rendered server-side by Rust. This component
 * adds client-side interactivity.
 */
@customElement('stencila-nav-tree')
export class StencilaNavTree extends LitElement {
  /**
   * Whether groups are collapsible
   */
  @property({ type: Boolean })
  collapsible = true

  /**
   * Default expansion state for collapsible groups
   */
  @property({ type: String })
  expanded: 'all' | 'none' | 'first-level' | 'current-path' = 'all'

  /**
   * Whether to scroll the active item into view on page load
   */
  @property({ type: Boolean, attribute: 'scroll-to-active' })
  scrollToActive = true

  /**
   * Use Light DOM so theme CSS can style the component
   */
  protected override createRenderRoot() {
    return this
  }

  override connectedCallback() {
    super.connectedCallback()
    this.setupToggleListeners()

    if (this.scrollToActive) {
      // Use requestAnimationFrame to ensure DOM is ready
      requestAnimationFrame(() => {
        this.scrollActiveIntoView()
      })
    }

    // Listen for SPA navigation to update active state
    window.addEventListener(GlideEvents.END, this.handleGlideEnd)
  }

  override disconnectedCallback() {
    super.disconnectedCallback()
    window.removeEventListener(GlideEvents.END, this.handleGlideEnd)
  }

  /**
   * Set up click listeners on all toggle buttons
   */
  private setupToggleListeners() {
    const toggleButtons = this.querySelectorAll<HTMLButtonElement>('.toggle')

    for (const button of toggleButtons) {
      button.addEventListener('click', this.handleToggleClick)
    }

    // Add keyboard navigation to the nav element
    const nav = this.querySelector('nav')
    if (nav) {
      nav.addEventListener('keydown', this.handleKeydown)
    }
  }

  /**
   * Handle toggle button click to expand/collapse group
   */
  private handleToggleClick = (event: Event) => {
    const button = event.currentTarget as HTMLButtonElement
    const groupItem = button.closest('.item[data-type="group"]')

    if (!groupItem) {
      return
    }

    const isExpanded = groupItem.getAttribute('data-expanded') === 'true'
    const newExpanded = !isExpanded

    // Update data attribute and ARIA state
    groupItem.setAttribute('data-expanded', String(newExpanded))
    groupItem.setAttribute('aria-expanded', String(newExpanded))
    button.setAttribute('aria-expanded', String(newExpanded))
  }

  /**
   * Handle Glide navigation end - update active state
   */
  private handleGlideEnd = () => {
    // Small delay to ensure DOM is updated
    requestAnimationFrame(() => {
      this.updateActiveState()

      // Re-expand groups in current path if using that mode
      if (this.expanded === 'current-path') {
        this.expandCurrentPath()
      }

      if (this.scrollToActive) {
        this.scrollActiveIntoView()
      }
    })
  }

  /**
   * Update the active state based on current URL
   */
  private updateActiveState() {
    const currentPath = window.location.pathname

    // Remove active state from all items
    const activeItems = this.querySelectorAll('[data-active="true"]')
    for (const item of activeItems) {
      item.setAttribute('data-active', 'false')
      const link = item.querySelector('a')
      if (link) {
        link.removeAttribute('aria-current')
      }
    }

    // Find and activate the matching link
    const links = this.querySelectorAll<HTMLAnchorElement>('a[href]')
    for (const link of links) {
      const href = link.getAttribute('href')
      if (href === currentPath) {
        const item = link.closest('.item')
        if (item) {
          item.setAttribute('data-active', 'true')
          link.setAttribute('aria-current', 'page')
        }
        break
      }
    }
  }

  /**
   * Expand all groups that contain the current page
   */
  private expandCurrentPath() {
    // Find the active item
    const activeItem = this.querySelector('[data-active="true"]')
    if (!activeItem) {
      return
    }

    // Walk up the tree and expand all parent groups
    let current: Element | null = activeItem.parentElement
    while (current && current !== this) {
      if (
        current.classList.contains('item') &&
        current.getAttribute('data-type') === 'group'
      ) {
        current.setAttribute('data-expanded', 'true')
        current.setAttribute('aria-expanded', 'true')

        // Also update the toggle button
        const toggle = current.querySelector('.toggle')
        if (toggle) {
          toggle.setAttribute('aria-expanded', 'true')
        }
      }
      current = current.parentElement
    }
  }

  /**
   * Scroll the nav container to make the active link visible
   */
  private scrollActiveIntoView() {
    const activeItem = this.querySelector('[data-active="true"]')
    if (!activeItem) {
      return
    }

    const activeLink = activeItem.querySelector('a')
    if (!activeLink) {
      return
    }

    // Find the scrollable container (sidebar content or this component)
    const container = this.closest('.sidebar-content') || this

    // Check if link is outside the visible area of the container
    const containerRect = container.getBoundingClientRect()
    const linkRect = activeLink.getBoundingClientRect()

    if (linkRect.top < containerRect.top || linkRect.bottom > containerRect.bottom) {
      activeLink.scrollIntoView({ block: 'center', behavior: 'smooth' })
    }
  }

  /**
   * Handle keyboard navigation within the nav tree
   */
  private handleKeydown = (event: KeyboardEvent) => {
    // Get all focusable elements (links and toggle buttons)
    const focusableSelector =
      'a[href], button.toggle:not([disabled])'
    const focusables = Array.from(
      this.querySelectorAll<HTMLElement>(focusableSelector)
    ).filter(
      (el) => el.offsetParent !== null // Only visible elements
    )

    const currentIndex = focusables.findIndex((el) => el === document.activeElement)

    if (currentIndex === -1) {
      return
    }

    const currentElement = focusables[currentIndex]
    let newIndex: number | null = null

    switch (event.key) {
      case 'ArrowDown':
        newIndex = Math.min(currentIndex + 1, focusables.length - 1)
        break

      case 'ArrowUp':
        newIndex = Math.max(currentIndex - 1, 0)
        break

      case 'ArrowRight': {
        // If on a collapsed group toggle, expand it
        const groupItem = currentElement.closest('.item[data-type="group"]')
        if (
          groupItem &&
          groupItem.getAttribute('data-expanded') === 'false' &&
          currentElement.classList.contains('toggle')
        ) {
          event.preventDefault()
          currentElement.click()
        }
        return
      }

      case 'ArrowLeft': {
        // If on an expanded group toggle, collapse it
        const groupItem = currentElement.closest('.item[data-type="group"]')
        if (
          groupItem &&
          groupItem.getAttribute('data-expanded') === 'true' &&
          currentElement.classList.contains('toggle')
        ) {
          event.preventDefault()
          currentElement.click()
          return
        }

        // Otherwise, move focus to parent group's toggle
        const parentGroup = currentElement.closest('.children')?.closest(
          '.item[data-type="group"]'
        )
        if (parentGroup) {
          const parentToggle =
            parentGroup.querySelector<HTMLButtonElement>('.toggle')
          if (parentToggle) {
            event.preventDefault()
            parentToggle.focus()
          }
        }
        return
      }

      case 'Home':
        newIndex = 0
        break

      case 'End':
        newIndex = focusables.length - 1
        break

      default:
        return
    }

    if (newIndex !== null && newIndex !== currentIndex) {
      event.preventDefault()
      focusables[newIndex].focus()
    }
  }

  /**
   * Reinitialize the nav tree after SPA navigation
   * Called by glide.ts after page transitions
   */
  reinitialize() {
    this.updateActiveState()

    if (this.expanded === 'current-path') {
      this.expandCurrentPath()
    }

    if (this.scrollToActive) {
      this.scrollActiveIntoView()
    }
  }

  /**
   * Update active link based on URL
   * Called by glide.ts after page transitions
   */
  updateActiveLink(_url: string) {
    this.updateActiveState()

    if (this.expanded === 'current-path') {
      this.expandCurrentPath()
    }

    if (this.scrollToActive) {
      this.scrollActiveIntoView()
    }
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'stencila-nav-tree': StencilaNavTree
  }
}
