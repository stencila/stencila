import { html, LitElement, nothing, PropertyValues } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { GlideEvents } from '../glide/events'

import './color-mode'

/**
 * Site layout shell component
 *
 * Uses Light DOM so that theme CSS applies directly to layout elements.
 * Manages responsive sidebar state for collapsible sidebars at mobile breakpoints.
 */
@customElement('stencila-layout')
export class StencilaLayout extends LitElement {
  /**
   * Whether left sidebar is collapsible (default: true)
   * Set via attribute from server-rendered config
   */
  @property({ type: Boolean, attribute: 'left-sidebar-collapsible' })
  leftSidebarCollapsible = true

  /**
   * Whether right sidebar is collapsible (default: true)
   */
  @property({ type: Boolean, attribute: 'right-sidebar-collapsible' })
  rightSidebarCollapsible = true

  /**
   * Breakpoint for sidebar collapse in pixels (default: 1024)
   */
  @property({ type: Number, attribute: 'collapse-breakpoint' })
  collapseBreakpoint = 1024

  /**
   * Left sidebar open state
   */
  @state()
  private leftSidebarOpen = false

  /**
   * Right sidebar open state
   */
  @state()
  private rightSidebarOpen = false

  /**
   * Whether we're currently in collapsed/mobile mode
   */
  @state()
  private isCollapsedMode = false

  /**
   * Media query list for responsive breakpoint
   */
  private mediaQuery: MediaQueryList | null = null

  /**
   * Track which sidebar was last opened (for focus restoration)
   */
  private lastOpenedSide: 'left' | 'right' | null = null

  /**
   * Override to use Light DOM so theme CSS applies
   */
  protected override createRenderRoot() {
    return this
  }

  override connectedCallback() {
    super.connectedCallback()
    this.setupMediaQuery()
    document.addEventListener('keydown', this.handleKeydown)
    // Auto-close sidebars after SPA navigation completes
    window.addEventListener(GlideEvents.END, this.handleGlideEnd)
  }

  override disconnectedCallback() {
    super.disconnectedCallback()
    this.mediaQuery?.removeEventListener('change', this.handleMediaQueryChange)
    document.removeEventListener('keydown', this.handleKeydown)
    window.removeEventListener(GlideEvents.END, this.handleGlideEnd)
  }

  override updated(changedProperties: PropertyValues) {
    super.updated(changedProperties)

    // Re-setup media query if breakpoint changes
    if (changedProperties.has('collapseBreakpoint')) {
      this.setupMediaQuery()
    }
  }

  /**
   * Set up media query listener for responsive behavior
   */
  private setupMediaQuery() {
    // Remove old listener if exists
    this.mediaQuery?.removeEventListener('change', this.handleMediaQueryChange)

    this.mediaQuery = window.matchMedia(
      `(max-width: ${this.collapseBreakpoint}px)`
    )
    this.isCollapsedMode = this.mediaQuery.matches
    this.mediaQuery.addEventListener('change', this.handleMediaQueryChange)
  }

  private handleMediaQueryChange = (e: MediaQueryListEvent) => {
    this.isCollapsedMode = e.matches
    // Close sidebars when switching back to desktop
    if (!e.matches) {
      this.closeSidebars()
    }
  }

  /**
   * Handle keyboard events - Escape closes sidebars
   */
  private handleKeydown = (e: KeyboardEvent) => {
    if (e.key === 'Escape' && (this.leftSidebarOpen || this.rightSidebarOpen)) {
      this.closeSidebars()
    }
  }

  /**
   * Handle Glide navigation end - auto-close sidebars on mobile
   */
  private handleGlideEnd = () => {
    if (this.isCollapsedMode && (this.leftSidebarOpen || this.rightSidebarOpen)) {
      this.closeSidebars()
    }
  }

  /**
   * Toggle left sidebar
   */
  toggleLeftSidebar() {
    if (!this.leftSidebarCollapsible) return

    this.leftSidebarOpen = !this.leftSidebarOpen
    this.rightSidebarOpen = false // Close other sidebar

    if (this.leftSidebarOpen) {
      this.lastOpenedSide = 'left'
    }

    this.updateSidebarAttributes()
    this.manageFocus('left')
  }

  /**
   * Toggle right sidebar
   */
  toggleRightSidebar() {
    if (!this.rightSidebarCollapsible) return

    this.rightSidebarOpen = !this.rightSidebarOpen
    this.leftSidebarOpen = false // Close other sidebar

    if (this.rightSidebarOpen) {
      this.lastOpenedSide = 'right'
    }

    this.updateSidebarAttributes()
    this.manageFocus('right')
  }

  /**
   * Close all sidebars
   */
  closeSidebars() {
    const wasOpen = this.leftSidebarOpen || this.rightSidebarOpen
    const sideToFocus = this.lastOpenedSide

    this.leftSidebarOpen = false
    this.rightSidebarOpen = false
    this.updateSidebarAttributes()

    // Return focus to the toggle button that opened the sidebar
    if (wasOpen && sideToFocus) {
      // Wait for render to complete, then focus toggle
      this.updateComplete.then(() => {
        const toggleSelector = `.sidebar-toggle-${sideToFocus}`
        const toggle = this.querySelector<HTMLElement>(toggleSelector)
        toggle?.focus()
      })
    }
  }

  /**
   * Update data attributes on host element for CSS targeting
   */
  private updateSidebarAttributes() {
    if (this.leftSidebarOpen) {
      this.setAttribute('data-left-sidebar-open', '')
    } else {
      this.removeAttribute('data-left-sidebar-open')
    }

    if (this.rightSidebarOpen) {
      this.setAttribute('data-right-sidebar-open', '')
    } else {
      this.removeAttribute('data-right-sidebar-open')
    }
  }

  /**
   * Manage focus when sidebar opens/closes
   */
  private manageFocus(side: 'left' | 'right') {
    const isOpen =
      side === 'left' ? this.leftSidebarOpen : this.rightSidebarOpen
    const selector =
      side === 'left' ? 'stencila-left-sidebar' : 'stencila-right-sidebar'

    if (isOpen) {
      const sidebar = this.querySelector(selector)
      if (!sidebar) return

      // Focus first focusable element after transition completes
      const focusFirstElement = () => {
        const firstFocusable = sidebar.querySelector<HTMLElement>(
          'a[href], button:not([disabled]), input:not([disabled]), [tabindex]:not([tabindex="-1"])'
        )
        firstFocusable?.focus()
        sidebar.removeEventListener('transitionend', onTransitionEnd)
      }

      const onTransitionEnd = (e: TransitionEvent) => {
        // Only respond to transform transition on the sidebar itself
        if (e.target === sidebar && e.propertyName === 'transform') {
          focusFirstElement()
        }
      }

      sidebar.addEventListener('transitionend', onTransitionEnd)

      // Fallback: if no transition occurs (e.g., reduced motion), focus immediately
      // Check after a frame to allow transition to start
      requestAnimationFrame(() => {
        const style = getComputedStyle(sidebar)
        const duration = parseFloat(style.transitionDuration) || 0
        if (duration === 0) {
          focusFirstElement()
        }
      })
    }
  }

  /**
   * Handle overlay click - close sidebars
   */
  private handleOverlayClick = () => {
    this.closeSidebars()
  }

  protected override render() {
    // Only render toggle buttons and overlay when in collapsed mode
    if (!this.isCollapsedMode) {
      return nothing
    }

    const hasLeftSidebar = this.hasAttribute('left-sidebar')
    const hasRightSidebar = this.hasAttribute('right-sidebar')

    return html`
      ${hasLeftSidebar && this.leftSidebarCollapsible
        ? html`
            <button
              class="sidebar-toggle sidebar-toggle-left"
              @click=${this.toggleLeftSidebar}
              aria-expanded=${this.leftSidebarOpen}
              aria-label=${this.leftSidebarOpen
                ? 'Close navigation sidebar'
                : 'Open navigation sidebar'}
            >
              <span class="toggle-icon"></span>
            </button>
          `
        : nothing}
      ${hasRightSidebar && this.rightSidebarCollapsible
        ? html`
            <button
              class="sidebar-toggle sidebar-toggle-right"
              @click=${this.toggleRightSidebar}
              aria-expanded=${this.rightSidebarOpen}
              aria-label=${this.rightSidebarOpen
                ? 'Close table of contents'
                : 'Open table of contents'}
            >
              <span class="toggle-icon"></span>
            </button>
          `
        : nothing}

      <div
        class="sidebar-overlay"
        @click=${this.handleOverlayClick}
        aria-hidden="true"
      ></div>
    `
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'stencila-layout': StencilaLayout
  }
}
