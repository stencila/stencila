import { LitElement } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import './header' // Header component
import './nav-tabs' // Navigation tabs component
import './nav-tree' // Navigation tree component
import './footer' // Footer component
import './breadcrumbs' // Breadcrumbs component
import './page-nav' // Page navigation component

/**
 * Site layout shell component
 *
 * A CSS Grid-based layout container for site pages with slots for:
 *
 * - header: Site header with logo, navigation tabs, icons
 * - left-sidebar: Navigation tree
 * - right-sidebar: Table of contents
 * - content: Main page content (the document view)
 * - footer: Site footer with links and copyright
 *
 * Uses Light DOM so that theme CSS applies directly to layout elements.
 * The layout is responsive and hides sidebars at smaller breakpoints.
 */
@customElement('stencila-layout')
export class StencilaLayout extends LitElement {
  /**
   * Whether the left sidebar is enabled
   */
  @property({ type: Boolean, attribute: 'left-sidebar' })
  leftSidebar: boolean = false

  /**
   * Whether the right sidebar is enabled
   */
  @property({ type: Boolean, attribute: 'right-sidebar' })
  rightSidebar: boolean = false

  /**
   * Whether the mobile sidebar is currently open
   */
  @property({ type: Boolean, reflect: true, attribute: 'mobile-sidebar-open' })
  mobileSidebarOpen: boolean = false

  private mobileNavToggle: HTMLButtonElement | null = null

  /**
   * Override to use Light DOM so theme CSS applies
   */
  protected override createRenderRoot() {
    return this
  }

  override connectedCallback() {
    super.connectedCallback()

    // Set attributes for CSS grid layout based on sidebar state
    this.updateLayoutAttributes()

    // Set up mobile nav toggle button
    this.mobileNavToggle = this.querySelector('.mobile-nav-toggle')
    if (this.mobileNavToggle) {
      this.mobileNavToggle.addEventListener('click', this.handleMobileNavToggleClick)
    }

    // Listen for escape key to close mobile sidebar
    this.addEventListener('keydown', this.handleKeydown)
  }

  override disconnectedCallback() {
    super.disconnectedCallback()

    if (this.mobileNavToggle) {
      this.mobileNavToggle.removeEventListener('click', this.handleMobileNavToggleClick)
    }

    this.removeEventListener('keydown', this.handleKeydown)
  }

  override updated(changedProperties: Map<string, unknown>) {
    if (
      changedProperties.has('leftSidebar') ||
      changedProperties.has('rightSidebar')
    ) {
      this.updateLayoutAttributes()
    }

    // Update mobile nav toggle aria-expanded state
    if (changedProperties.has('mobileSidebarOpen') && this.mobileNavToggle) {
      this.mobileNavToggle.setAttribute('aria-expanded', String(this.mobileSidebarOpen))
    }
  }

  /**
   * Update attributes used by CSS for grid layout
   */
  private updateLayoutAttributes() {
    if (!this.leftSidebar) {
      this.setAttribute('no-left-sidebar', '')
    } else {
      this.removeAttribute('no-left-sidebar')
    }

    if (!this.rightSidebar) {
      this.setAttribute('no-right-sidebar', '')
    } else {
      this.removeAttribute('no-right-sidebar')
    }
  }

  /**
   * Handle keyboard events for accessibility
   */
  private handleKeydown = (event: KeyboardEvent) => {
    if (event.key === 'Escape' && this.mobileSidebarOpen) {
      this.closeMobileSidebar()
    }
  }

  /**
   * Handle mobile nav toggle button click
   */
  private handleMobileNavToggleClick = () => {
    this.toggleMobileSidebar()
  }

  /**
   * Toggle the mobile sidebar open/closed
   */
  toggleMobileSidebar() {
    this.mobileSidebarOpen = !this.mobileSidebarOpen
    if (this.mobileSidebarOpen) {
      this.classList.add('mobile-sidebar-open')
    } else {
      this.classList.remove('mobile-sidebar-open')
    }
  }

  /**
   * Close the mobile sidebar
   */
  closeMobileSidebar() {
    this.mobileSidebarOpen = false
    this.classList.remove('mobile-sidebar-open')
  }

  /**
   * Open the mobile sidebar
   */
  openMobileSidebar() {
    this.mobileSidebarOpen = true
    this.classList.add('mobile-sidebar-open')
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'stencila-layout': StencilaLayout
  }
}
