import { LitElement } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { GlideEvents } from '../glide/events'
import {
  addAccessBadgeToItem,
  canAccess,
  getAccessConfig,
} from '../utils/access'

import type {
  AccessLevel,
  RouteAccessConfig,
  SiteAuthStatusResponse,
} from './site-action/types'
import { getCachedAuthStatus, isLocalhost } from './site-action/utils'

/**
 * Navigation tree component
 *
 * Provides interactivity for site navigation:
 * - Collapsible groups with toggle buttons
 * - Active page highlighting with SPA navigation support
 * - Keyboard navigation (arrow keys)
 * - Auto-scroll to show active item
 * - Access-aware filtering (hide/show restricted items)
 *
 * The HTML structure is rendered server-side by Rust. This component
 * adds client-side interactivity and access-based filtering.
 */
@customElement('stencila-nav-tree')
export class StencilaNavTree extends LitElement {
  /**
   * Whether groups are collapsible
   */
  @property({ type: Boolean })
  collapsible = true

  /**
   * How deep to expand groups by default
   *
   * - undefined = all groups expanded (unlimited)
   * - 0 = all groups collapsed
   * - 1 = only top-level groups expanded
   * - 3 = groups expanded up to level 3
   */
  @property({ type: Number, attribute: 'expand-depth' })
  expandDepth?: number

  /**
   * Whether to expand groups containing the current page
   *
   * When true, groups that are ancestors of the current page are
   * expanded regardless of expand-depth.
   */
  @property({ type: Boolean, attribute: 'expand-current' })
  expandCurrent = true

  /**
   * Whether to scroll the active item into view on page load
   */
  @property({ type: Boolean, attribute: 'scroll-to-active' })
  scrollToActive = true

  /**
   * How to handle items requiring higher access than user has
   * - 'hide': Hide items user cannot access (default)
   * - 'show': Show all items (restricted items remain visible)
   */
  @property({ type: String, attribute: 'restricted-items' })
  restrictedItems: 'hide' | 'show' = 'hide'

  /**
   * Whether to show access badges on items that are restricted
   * Badges indicate the required access level (password, team, etc.)
   */
  @property({ type: Boolean, attribute: 'show-access-badges' })
  showAccessBadges = true

  /**
   * Access configuration (from _access.json)
   */
  @state()
  private accessConfig: RouteAccessConfig | undefined = undefined

  /**
   * User's access level from auth status
   */
  @state()
  private userAccessLevel: AccessLevel = 'public'

  /**
   * Use Light DOM so theme CSS can style the component
   */
  protected override createRenderRoot() {
    return this
  }

  override connectedCallback() {
    super.connectedCallback()
    this.setupToggleListeners()

    // Load access configuration and apply filtering
    this.loadAccessAndFilter()

    if (this.scrollToActive) {
      // Use requestAnimationFrame to ensure DOM is ready
      requestAnimationFrame(() => {
        this.scrollActiveIntoView()
      })
    }

    // Listen for SPA navigation to update active state
    window.addEventListener(GlideEvents.END, this.handleGlideEnd)
  }

  /**
   * Load access configuration and user access level, then apply filtering
   */
  private async loadAccessAndFilter() {
    try {
      // On localhost preview, grant team access (user has repo access)
      if (isLocalhost()) {
        this.userAccessLevel = 'team'
        // Still fetch access config to show badges
        this.accessConfig = await getAccessConfig()
        if (this.accessConfig) {
          this.applyAccessFiltering()
        }
        return
      }

      // Fetch access config and auth status in parallel
      const [config, authStatus] = await Promise.all([
        getAccessConfig(),
        this.fetchAuthStatus(),
      ])

      this.accessConfig = config
      this.userAccessLevel = authStatus?.userAccessLevel ?? 'public'

      // Apply access filtering if we have restrictions
      if (config) {
        this.applyAccessFiltering()
      }
    } catch (error) {
      console.warn('[StencilaNavTree] Failed to load access config:', error)
    }
  }

  /**
   * Fetch auth status to get user's access level
   */
  private async fetchAuthStatus(): Promise<SiteAuthStatusResponse | null> {
    try {
      // Use the cached auth status fetcher
      return await getCachedAuthStatus(
        '',
        '/__stencila/auth/status',
        async () => ({})
      )
    } catch {
      return null
    }
  }

  /**
   * Apply access-based filtering to nav items
   *
   * Hides items the user cannot access (if restrictedItems='hide')
   * and adds badges to restricted items (if showAccessBadges=true).
   */
  private applyAccessFiltering() {
    if (!this.accessConfig) {
      return
    }

    const items = this.querySelectorAll<HTMLElement>('.item[data-access]')

    for (const item of items) {
      const requiredLevel = item.dataset.access as AccessLevel
      const userCanAccess = canAccess(this.userAccessLevel, requiredLevel)

      if (!userCanAccess && this.restrictedItems === 'hide') {
        // Hide items user cannot access
        item.style.display = 'none'
      } else if (this.showAccessBadges) {
        // Add badge to indicate access restriction
        addAccessBadgeToItem(item, requiredLevel, 'a, .group-link, .label')
      }
    }
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

      // Re-expand groups in current path if expand-current is enabled
      if (this.expandCurrent) {
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
    let newIndex: number | null

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

    if (this.expandCurrent) {
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

    if (this.expandCurrent) {
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
