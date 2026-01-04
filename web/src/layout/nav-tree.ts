import { LitElement } from 'lit'
import { customElement } from 'lit/decorators.js'

/** Local storage key for persisted expansion state */
const STORAGE_KEY = 'stencila-nav-tree-expanded'

/**
 * Navigation tree component for site routes
 *
 * A Light DOM component that enhances the server-rendered navigation tree
 * with keyboard navigation and expand/collapse behavior.
 *
 * The HTML is server-rendered with ARIA tree pattern:
 * - ul[role="tree"] - root navigation list
 * - li[role="treeitem"] - navigation items
 * - ul[role="group"] - nested groups
 * - details/summary - collapsible groups
 *
 * This component adds:
 * - Keyboard navigation (arrow keys, Enter, Home/End)
 * - Auto-expansion of ancestor groups for the active item
 * - Persistent expansion state in local storage
 * - Focus management
 */
@customElement('stencila-nav-tree')
export class StencilaNavTree extends LitElement {
  /**
   * Override to use Light DOM so theme CSS applies
   */
  protected override createRenderRoot() {
    return this
  }

  override connectedCallback() {
    super.connectedCallback()

    // Restore expansion state from local storage
    this.restoreExpansionState()

    // Expand ancestors of the active item (overrides restored state for active path)
    this.expandActiveAncestors()

    // Listen for details toggle to persist state
    this.addEventListener('toggle', this.handleToggle, true)

    // Set up keyboard navigation
    this.addEventListener('keydown', this.handleKeydown)
  }

  override disconnectedCallback() {
    super.disconnectedCallback()
    this.removeEventListener('toggle', this.handleToggle, true)
    this.removeEventListener('keydown', this.handleKeydown)
  }

  /**
   * Expand all ancestor details elements for the active link
   */
  private expandActiveAncestors() {
    const activeLink = this.querySelector('.nav-link.active')
    if (!activeLink) return

    // Walk up the DOM and open all ancestor details elements
    let element = activeLink.parentElement
    while (element && element !== this) {
      if (element.tagName === 'DETAILS') {
        ;(element as HTMLDetailsElement).open = true
      }
      element = element.parentElement
    }
  }

  /**
   * Generate a deterministic ID for a details element
   *
   * Uses the text content of ancestor summaries to create a path-like ID.
   * For example: "Docs/Getting Started/Installation"
   */
  private getNodeId(details: HTMLDetailsElement): string {
    const parts: string[] = []

    // Get the summary text for this details
    const summary = details.querySelector(':scope > summary')
    if (summary) {
      parts.push(summary.textContent?.trim() ?? '')
    }

    // Walk up to get ancestor summaries
    let parent = details.parentElement?.closest('details') as HTMLDetailsElement | null
    while (parent) {
      const parentSummary = parent.querySelector(':scope > summary')
      if (parentSummary) {
        parts.unshift(parentSummary.textContent?.trim() ?? '')
      }
      parent = parent.parentElement?.closest('details') as HTMLDetailsElement | null
    }

    return parts.join('/')
  }

  /**
   * Get all details elements in this tree
   */
  private getAllDetails(): HTMLDetailsElement[] {
    return Array.from(this.querySelectorAll('details'))
  }

  /**
   * Load expansion state from local storage
   */
  private loadExpansionState(): Record<string, boolean> {
    try {
      const stored = localStorage.getItem(STORAGE_KEY)
      if (stored) {
        return JSON.parse(stored)
      }
    } catch {
      // Ignore parse errors
    }
    return {}
  }

  /**
   * Save expansion state to local storage
   */
  private saveExpansionState(state: Record<string, boolean>) {
    try {
      localStorage.setItem(STORAGE_KEY, JSON.stringify(state))
    } catch {
      // Ignore storage errors (e.g., quota exceeded)
    }
  }

  /**
   * Restore expansion state from local storage
   */
  private restoreExpansionState() {
    const state = this.loadExpansionState()
    if (Object.keys(state).length === 0) {
      return // No saved state, use server-rendered defaults
    }

    const details = this.getAllDetails()
    for (const detail of details) {
      const id = this.getNodeId(detail)
      if (id in state) {
        detail.open = state[id]
      }
    }
  }

  /**
   * Handle details toggle event to persist state
   */
  private handleToggle = (event: Event) => {
    const target = event.target as HTMLElement
    if (target.tagName !== 'DETAILS') return

    const details = target as HTMLDetailsElement
    const id = this.getNodeId(details)
    const state = this.loadExpansionState()
    state[id] = details.open
    this.saveExpansionState(state)
  }

  /**
   * Handle keyboard navigation
   */
  private handleKeydown = (event: KeyboardEvent) => {
    const target = event.target as HTMLElement
    if (!target) return

    // Only handle if focus is on a focusable nav element
    const isFocusable =
      target.matches('a.nav-link, summary.nav-group') ||
      target.closest('a.nav-link, summary.nav-group')

    if (!isFocusable) return

    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault()
        this.focusNextItem(target)
        break
      case 'ArrowUp':
        event.preventDefault()
        this.focusPreviousItem(target)
        break
      case 'ArrowRight':
        event.preventDefault()
        this.expandOrFocusChild(target)
        break
      case 'ArrowLeft':
        event.preventDefault()
        this.collapseOrFocusParent(target)
        break
      case 'Home':
        event.preventDefault()
        this.focusFirstItem()
        break
      case 'End':
        event.preventDefault()
        this.focusLastItem()
        break
      case 'Enter':
      case ' ':
        if (target.matches('summary.nav-group')) {
          // Let the default details toggle happen
        } else if (target.matches('a.nav-link')) {
          // Let the default link navigation happen
        }
        break
    }
  }

  /**
   * Get all visible focusable items in tree order
   */
  private getVisibleItems(): HTMLElement[] {
    const items: HTMLElement[] = []
    const walker = document.createTreeWalker(this, NodeFilter.SHOW_ELEMENT, {
      acceptNode: (node: Node) => {
        const el = node as HTMLElement
        // Include links and summary elements
        if (el.matches('a.nav-link, summary.nav-group')) {
          // Check if visible (not in a closed details)
          const details = el.closest('details')
          if (details && !details.open && el.tagName !== 'SUMMARY') {
            return NodeFilter.FILTER_SKIP
          }
          // If this is inside a group within a closed details, skip
          const parentDetails = el.parentElement?.closest('details')
          if (
            parentDetails &&
            !parentDetails.open &&
            !el.matches('summary.nav-group')
          ) {
            return NodeFilter.FILTER_SKIP
          }
          return NodeFilter.FILTER_ACCEPT
        }
        return NodeFilter.FILTER_SKIP
      },
    })

    let node: Node | null
    while ((node = walker.nextNode())) {
      items.push(node as HTMLElement)
    }
    return items
  }

  /**
   * Focus the next item in tree order
   */
  private focusNextItem(current: HTMLElement) {
    const items = this.getVisibleItems()
    const currentIndex = items.indexOf(current)
    if (currentIndex < items.length - 1) {
      items[currentIndex + 1].focus()
    }
  }

  /**
   * Focus the previous item in tree order
   */
  private focusPreviousItem(current: HTMLElement) {
    const items = this.getVisibleItems()
    const currentIndex = items.indexOf(current)
    if (currentIndex > 0) {
      items[currentIndex - 1].focus()
    }
  }

  /**
   * Focus the first item in the tree
   */
  private focusFirstItem() {
    const items = this.getVisibleItems()
    if (items.length > 0) {
      items[0].focus()
    }
  }

  /**
   * Focus the last item in the tree
   */
  private focusLastItem() {
    const items = this.getVisibleItems()
    if (items.length > 0) {
      items[items.length - 1].focus()
    }
  }

  /**
   * Expand a collapsed group or focus its first child
   */
  private expandOrFocusChild(current: HTMLElement) {
    if (current.matches('summary.nav-group')) {
      const details = current.closest('details')
      if (details && !details.open) {
        // Expand the group
        details.open = true
      } else if (details?.open) {
        // Focus first child
        const firstChild = details.querySelector(
          'ul[role="group"] > li > a.nav-link, ul[role="group"] > li > details > summary'
        ) as HTMLElement | null
        if (firstChild) {
          firstChild.focus()
        }
      }
    }
  }

  /**
   * Update the active link to match the current URL
   *
   * Called by the navigation module after a page swap to update
   * which link is marked as active and ensure its ancestors are expanded.
   *
   * @param url - The new URL to match against nav links
   */
  updateActiveLink(url: string) {
    // Parse URL to get pathname for matching
    const pathname = new URL(url, window.location.origin).pathname

    // Remove active class from old link
    const oldActive = this.querySelector('.nav-link.active')
    if (oldActive) {
      oldActive.classList.remove('active')
      oldActive.removeAttribute('aria-current')
    }

    // Find link matching the new URL
    const links = this.querySelectorAll<HTMLAnchorElement>('a.nav-link')
    for (const link of links) {
      const linkPathname = new URL(link.href, window.location.origin).pathname
      if (linkPathname === pathname) {
        link.classList.add('active')
        link.setAttribute('aria-current', 'page')

        // Expand ancestor details elements
        let element = link.parentElement
        while (element && element !== this) {
          if (element.tagName === 'DETAILS') {
            ;(element as HTMLDetailsElement).open = true
          }
          element = element.parentElement
        }

        // Scroll active link into view if needed
        link.scrollIntoView({ block: 'nearest', behavior: 'smooth' })
        break
      }
    }
  }

  /**
   * Collapse a group or focus the parent summary
   */
  private collapseOrFocusParent(current: HTMLElement) {
    if (current.matches('summary.nav-group')) {
      const details = current.closest('details')
      if (details?.open) {
        // Collapse the group
        details.open = false
      } else {
        // Focus parent group's summary if exists
        const parentDetails = details?.parentElement?.closest('details')
        const parentSummary = parentDetails?.querySelector(
          ':scope > summary'
        ) as HTMLElement | null
        if (parentSummary) {
          parentSummary.focus()
        }
      }
    } else if (current.matches('a.nav-link')) {
      // Focus the parent group's summary
      const parentDetails = current.closest('details')
      const parentSummary = parentDetails?.querySelector(
        ':scope > summary'
      ) as HTMLElement | null
      if (parentSummary) {
        parentSummary.focus()
      }
    }
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'stencila-nav-tree': StencilaNavTree
  }
}
