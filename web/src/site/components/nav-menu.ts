import { LitElement } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { GlideEvents } from '../glide/events'

/**
 * Navigation menu component
 *
 * Provides interactivity for the horizontal navigation menu:
 * - Dropdown panels with hover/click trigger modes
 * - Configurable open/close delays
 * - Responsive mobile menu with hamburger toggle
 * - Keyboard navigation (Escape to close, Tab through items)
 * - Active page highlighting with SPA navigation support
 *
 * The HTML structure is rendered server-side by Rust. This component
 * adds client-side interactivity.
 */
@customElement('stencila-nav-menu')
export class StencilaNavMenu extends LitElement {
  /**
   * How to render groups: auto|dropdowns|links
   */
  @property({ type: String })
  groups: 'auto' | 'dropdowns' | 'links' = 'auto'

  /**
   * Icon visibility: show|hide|dropdowns-only
   */
  @property({ type: String })
  icons: 'show' | 'hide' | 'dropdowns-only' = 'show'

  /**
   * Whether to show item descriptions in dropdowns
   */
  @property({ type: Boolean })
  descriptions = true

  /**
   * Dropdown trigger mode: hover|click
   */
  @property({ type: String })
  trigger: 'hover' | 'click' = 'hover'

  /**
   * Dropdown positioning style
   */
  @property({ type: String, attribute: 'dropdown-style' })
  dropdownStyle: 'full-width' | 'aligned' = 'full-width'

  /**
   * Delay in ms before opening dropdown on hover
   */
  @property({ type: Number, attribute: 'hover-delay' })
  hoverDelay = 150

  /**
   * Delay in ms before closing dropdown on mouse leave
   */
  @property({ type: Number, attribute: 'close-delay' })
  closeDelay = 300

  /**
   * Viewport width below which to show mobile menu
   */
  @property({ type: Number, attribute: 'mobile-breakpoint' })
  mobileBreakpoint = 1024

  /**
   * Whether mobile menu is expanded
   */
  @property({ type: Boolean, attribute: 'mobile-expanded', reflect: true })
  mobileExpanded = false

  /**
   * Whether we're in mobile mode
   */
  @state()
  private isMobileMode = false

  /**
   * Timeout for delayed dropdown open
   */
  private openTimeout: number | null = null

  /**
   * Timeout for delayed dropdown close
   */
  private closeTimeout: number | null = null

  /**
   * Currently open dropdown ID
   */
  private activeDropdownId: string | null = null

  /**
   * Media query for responsive breakpoint
   */
  private mediaQuery: MediaQueryList | null = null

  /**
   * Use Light DOM so theme CSS can style the component
   */
  protected override createRenderRoot() {
    return this
  }

  override connectedCallback() {
    super.connectedCallback()

    this.setupDropdownListeners()
    this.setupMobileMenu()
    this.setupMediaQuery()

    // Listen for SPA navigation
    window.addEventListener(GlideEvents.END, this.handleGlideEnd)

    // Listen for clicks outside to close dropdowns
    document.addEventListener('click', this.handleDocumentClick)

    // Listen for Escape key
    document.addEventListener('keydown', this.handleKeydown)
  }

  override disconnectedCallback() {
    super.disconnectedCallback()

    this.clearTimeouts()

    window.removeEventListener(GlideEvents.END, this.handleGlideEnd)
    document.removeEventListener('click', this.handleDocumentClick)
    document.removeEventListener('keydown', this.handleKeydown)

    if (this.mediaQuery) {
      this.mediaQuery.removeEventListener('change', this.handleMediaQueryChange)
    }
  }

  /**
   * Set up listeners on dropdown triggers
   */
  private setupDropdownListeners() {
    const triggers =
      this.querySelectorAll<HTMLButtonElement>('.trigger')

    for (const trigger of triggers) {
      if (this.trigger === 'hover') {
        // Hover mode - open on mouseenter, close on mouseleave with delays
        const item = trigger.closest('.item')
        if (item) {
          item.addEventListener('mouseenter', () =>
            this.handleMouseEnter(trigger)
          )
          item.addEventListener('mouseleave', () =>
            this.handleMouseLeave(trigger)
          )
        }
      }

      // Click always works (for keyboard/touch)
      trigger.addEventListener('click', () => this.handleTriggerClick(trigger))
    }
  }

  /**
   * Set up mobile menu toggle
   */
  private setupMobileMenu() {
    // Create and insert mobile toggle button if it doesn't exist
    if (!this.querySelector('.mobile-toggle')) {
      const nav = this.querySelector('nav')
      if (nav) {
        const toggle = document.createElement('button')
        toggle.className = 'mobile-toggle'
        toggle.setAttribute('aria-label', 'Toggle navigation menu')
        toggle.setAttribute('aria-expanded', 'false')
        toggle.innerHTML =
          '<span class="hamburger"><span></span><span></span><span></span></span>'
        toggle.addEventListener('click', this.handleMobileToggle)
        nav.insertBefore(toggle, nav.firstChild)
      }
    }
  }

  /**
   * Set up responsive media query
   */
  private setupMediaQuery() {
    this.mediaQuery = window.matchMedia(
      `(max-width: ${this.mobileBreakpoint}px)`
    )
    this.isMobileMode = this.mediaQuery.matches
    this.updateMobileModeAttribute()

    this.mediaQuery.addEventListener('change', this.handleMediaQueryChange)
  }

  /**
   * Handle media query change
   */
  private handleMediaQueryChange = (event: MediaQueryListEvent) => {
    this.isMobileMode = event.matches
    this.updateMobileModeAttribute()

    // Close dropdowns when switching modes
    this.closeAllDropdowns()

    // Close mobile menu when switching to desktop
    if (!this.isMobileMode) {
      this.mobileExpanded = false
      this.updateMobileToggleState()
    }
  }

  /**
   * Update data-mobile-mode attribute
   */
  private updateMobileModeAttribute() {
    this.setAttribute('data-mobile-mode', String(this.isMobileMode))
  }

  /**
   * Handle mobile toggle button click
   */
  private handleMobileToggle = () => {
    this.mobileExpanded = !this.mobileExpanded
    this.updateMobileToggleState()
    if (this.isMobileMode && this.mobileExpanded) {
      this.expandToCurrentRoute()
    }
  }

  /**
   * Update mobile toggle button state
   */
  private updateMobileToggleState() {
    const toggle = this.querySelector('.mobile-toggle')
    if (toggle) {
      toggle.setAttribute('aria-expanded', String(this.mobileExpanded))
    }
  }

  /**
   * Handle mouse enter on dropdown item (hover mode)
   */
  private handleMouseEnter(trigger: HTMLButtonElement) {
    if (this.isMobileMode) return

    this.clearTimeouts()

    // Close any other open dropdowns immediately
    this.closeAllDropdownsExcept(trigger)

    this.openTimeout = window.setTimeout(() => {
      this.openDropdown(trigger)
    }, this.hoverDelay)
  }

  /**
   * Handle mouse leave on dropdown item (hover mode)
   */
  private handleMouseLeave(trigger: HTMLButtonElement) {
    if (this.isMobileMode) return

    this.clearTimeouts()

    this.closeTimeout = window.setTimeout(() => {
      this.closeDropdown(trigger)
    }, this.closeDelay)
  }

  /**
   * Handle trigger button click
   */
  private handleTriggerClick(trigger: HTMLButtonElement) {
    const isExpanded = trigger.getAttribute('aria-expanded') === 'true'

    if (isExpanded) {
      this.closeDropdown(trigger)
    } else {
      // Close other dropdowns first (desktop behavior)
      if (!this.isMobileMode) {
        this.closeAllDropdowns()
      }
      this.openDropdown(trigger)
    }
  }

  /**
   * Open a dropdown
   */
  private openDropdown(trigger: HTMLButtonElement) {
    const item = trigger.closest('.item')
    if (!item) return

    const dropdownId = trigger.getAttribute('aria-controls')

    trigger.setAttribute('aria-expanded', 'true')
    item.setAttribute('data-dropdown-open', 'true')
    this.activeDropdownId = dropdownId
  }

  /**
   * Close a dropdown
   */
  private closeDropdown(trigger: HTMLButtonElement) {
    const item = trigger.closest('.item')
    if (!item) return

    trigger.setAttribute('aria-expanded', 'false')
    item.setAttribute('data-dropdown-open', 'false')

    const dropdownId = trigger.getAttribute('aria-controls')
    if (this.activeDropdownId === dropdownId) {
      this.activeDropdownId = null
    }
  }

  /**
   * Close all open dropdowns
   */
  private closeAllDropdowns() {
    const triggers = this.querySelectorAll<HTMLButtonElement>('.trigger')
    for (const trigger of triggers) {
      this.closeDropdown(trigger)
    }
  }

  /**
   * Close all dropdowns except the specified one
   */
  private closeAllDropdownsExcept(exceptTrigger: HTMLButtonElement) {
    const triggers = this.querySelectorAll<HTMLButtonElement>('.trigger')
    for (const trigger of triggers) {
      if (trigger !== exceptTrigger) {
        this.closeDropdown(trigger)
      }
    }
  }

  /**
   * Clear pending timeouts
   */
  private clearTimeouts() {
    if (this.openTimeout) {
      window.clearTimeout(this.openTimeout)
      this.openTimeout = null
    }
    if (this.closeTimeout) {
      window.clearTimeout(this.closeTimeout)
      this.closeTimeout = null
    }
  }

  /**
   * Handle document click - close dropdowns when clicking outside
   */
  private handleDocumentClick = (event: MouseEvent) => {
    const target = event.target as Element

    // Check if click is inside this nav menu
    if (this.contains(target)) {
      return
    }

    // Close all dropdowns
    this.closeAllDropdowns()

    // Close mobile menu
    if (this.isMobileMode && this.mobileExpanded) {
      this.mobileExpanded = false
      this.updateMobileToggleState()
    }
  }

  /**
   * Handle keyboard events
   */
  private handleKeydown = (event: KeyboardEvent) => {
    if (event.key === 'Escape') {
      this.closeAllDropdowns()

      // Close mobile menu
      if (this.isMobileMode && this.mobileExpanded) {
        this.mobileExpanded = false
        this.updateMobileToggleState()
      }
    }
  }

  /**
   * Handle Glide navigation end - update active state
   */
  private handleGlideEnd = () => {
    requestAnimationFrame(() => {
      this.updateActiveState()

      // Close mobile menu after navigation
      if (this.isMobileMode && this.mobileExpanded) {
        this.mobileExpanded = false
        this.updateMobileToggleState()
      }
    })
  }

  /**
   * Normalize a pathname by removing trailing slash (except for root "/")
   */
  private normalizePathname(path: string): string {
    return path.length > 1 && path.endsWith('/') ? path.slice(0, -1) : path
  }

  /**
   * Update active state based on current URL
   */
  private updateActiveState() {
    const currentPath = this.normalizePathname(window.location.pathname)

    // Remove aria-current from all links
    const allLinks = this.querySelectorAll<HTMLAnchorElement>('a[aria-current]')
    for (const link of allLinks) {
      link.removeAttribute('aria-current')
    }

    // Find and mark the matching link
    const links = this.querySelectorAll<HTMLAnchorElement>('a[href]')
    for (const link of links) {
      const href = link.getAttribute('href')
      if (href && this.normalizePathname(href) === currentPath) {
        link.setAttribute('aria-current', 'page')
        break
      }
    }
  }

  /**
   * Expand the mobile accordion to the current route.
   */
  private expandToCurrentRoute() {
    if (!this.isMobileMode) return

    const currentPath = this.normalizePathname(window.location.pathname)
    const links = this.querySelectorAll<HTMLAnchorElement>('a[href]')
    let activeLink: HTMLAnchorElement | null = null

    for (const link of links) {
      const href = link.getAttribute('href')
      if (href && this.normalizePathname(href) === currentPath) {
        activeLink = link
        break
      }
    }

    if (!activeLink) return

    const parentItem = activeLink.closest<HTMLLIElement>('li.item')
    if (!parentItem) return

    const trigger = parentItem.querySelector<HTMLButtonElement>('.trigger')
    if (trigger) {
      this.openDropdown(trigger)
    }
  }

  /**
   * Reinitialize the nav menu after SPA navigation
   * Called by glide.ts after page transitions
   */
  reinitialize() {
    this.updateActiveState()
  }

  /**
   * Update active link based on URL
   * Called by glide.ts after page transitions
   */
  updateActiveLink(_url: string) {
    this.updateActiveState()
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'stencila-nav-menu': StencilaNavMenu
  }
}
