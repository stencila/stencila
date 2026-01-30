import { LitElement, html, nothing } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { GlideEvents } from '../../glide/events'
import {
  SITE_ACTION_REGISTER,
  SITE_ACTION_BADGE_UPDATE,
  SITE_ACTION_UNREGISTER,
  SITE_ACTION_REQUEST_REGISTER,
  type ActionRegistration,
  type BadgeUpdateDetail,
} from '../site-action'

/**
 * Site actions zone component
 *
 * Consolidates multiple FABs (reviews, uploads, etc.) into a unified zone.
 * Uses event-based self-registration: child components dispatch events to register
 * themselves, and this component manages the FAB display and badge counts.
 *
 * Supports two modes:
 * - Collapsed (default): Main FAB expands on click to reveal action buttons
 * - Expanded: All action buttons always visible
 */
@customElement('stencila-site-actions')
export class StencilaSiteActions extends LitElement {
  /**
   * Position of the actions zone on the page
   */
  @property({ type: String, reflect: true })
  position: 'bottom-right' | 'bottom-left' | 'top-right' | 'top-left' =
    'bottom-right'

  /**
   * Direction for action buttons to expand
   */
  @property({ type: String })
  direction: 'vertical' | 'horizontal' = 'vertical'

  /**
   * Display mode for the actions zone
   */
  @property({ type: String, reflect: true })
  mode: 'collapsed' | 'expanded' = 'collapsed'

  /**
   * Comma-separated list of action IDs enabled for this route
   * Actions not in this list will be hidden
   */
  @property({ type: String })
  enabled: string = ''

  /**
   * Whether the collapsed FAB is expanded
   */
  @state()
  private expanded = false

  /**
   * Set of action IDs that are enabled for this route
   */
  @state()
  private enabledSet: Set<string> = new Set()

  /**
   * Registered actions from child components
   */
  @state()
  private actions: Map<string, ActionRegistration> = new Map()

  /**
   * Badge counts for each action
   */
  @state()
  private badges: Map<string, number> = new Map()

  /**
   * Use Light DOM so theme CSS can style the component
   */
  protected override createRenderRoot() {
    return this
  }

  override connectedCallback() {
    super.connectedCallback()

    // Parse initial enabled attribute
    this.parseEnabledAttribute()

    // Add event listeners for self-registration
    this.addEventListener(SITE_ACTION_REGISTER, this.handleRegister as EventListener)
    this.addEventListener(SITE_ACTION_BADGE_UPDATE, this.handleBadgeUpdate as EventListener)
    this.addEventListener(SITE_ACTION_UNREGISTER, this.handleUnregister as EventListener)

    // Global event listeners
    document.addEventListener('keydown', this.handleKeyDown)
    document.addEventListener('click', this.handleDocumentClick)

    // Listen for glide navigation to update visibility
    window.addEventListener(GlideEvents.END, this.handleGlideEnd)

    // Fallback: scan for children that may have connected before listeners attached
    requestAnimationFrame(() => this.scanForUnregisteredChildren())
  }

  override disconnectedCallback() {
    // Remove self-registration event listeners
    this.removeEventListener(SITE_ACTION_REGISTER, this.handleRegister as EventListener)
    this.removeEventListener(SITE_ACTION_BADGE_UPDATE, this.handleBadgeUpdate as EventListener)
    this.removeEventListener(SITE_ACTION_UNREGISTER, this.handleUnregister as EventListener)

    // Remove global event listeners
    document.removeEventListener('keydown', this.handleKeyDown)
    document.removeEventListener('click', this.handleDocumentClick)
    window.removeEventListener(GlideEvents.END, this.handleGlideEnd)

    super.disconnectedCallback()
  }

  // =========================================================================
  // Event Handlers for Self-Registration
  // =========================================================================

  /**
   * Handle action registration from child components
   */
  private handleRegister = (e: CustomEvent<ActionRegistration>) => {
    e.stopPropagation()

    this.actions.set(e.detail.id, e.detail)
    this.actions = new Map(this.actions) // trigger reactivity

    // Set position and hide-fab on child (always, regardless of mode)
    const child = e.target as HTMLElement
    if (child) {
      child.setAttribute('position', this.position)
      child.setAttribute('hide-fab', '')

      // Apply visibility based on enabled set
      const isVisible =
        this.enabledSet.size === 0 || this.enabledSet.has(e.detail.id)
      if (isVisible) {
        child.removeAttribute('data-action-hidden')
        child.style.display = ''
      } else {
        child.setAttribute('data-action-hidden', '')
        child.style.display = 'none'
      }
    }

    console.log(`[SiteActions] Registered action: ${e.detail.id}`)
  }

  /**
   * Handle badge count updates from child components
   */
  private handleBadgeUpdate = (e: CustomEvent<BadgeUpdateDetail>) => {
    e.stopPropagation()

    this.badges.set(e.detail.id, e.detail.count)
    this.badges = new Map(this.badges)
  }

  /**
   * Handle action unregistration when child disconnects
   */
  private handleUnregister = (e: CustomEvent<{ id: string }>) => {
    e.stopPropagation()

    this.actions.delete(e.detail.id)
    this.badges.delete(e.detail.id)
    this.actions = new Map(this.actions)
    this.badges = new Map(this.badges)

    console.log(`[SiteActions] Unregistered action: ${e.detail.id}`)
  }

  /**
   * Fallback scan for timing edge cases
   * Note: Children listen for this event in connectedCallback (before firstUpdated)
   * so they're ready to respond even if they haven't auto-registered yet
   */
  private scanForUnregisteredChildren() {
    const children = this.querySelectorAll('[data-site-action]')
    children.forEach((child) => {
      const id = child.getAttribute('data-site-action')
      if (id && !this.actions.has(id)) {
        // Request child to re-register using the defined constant
        child.dispatchEvent(new CustomEvent(SITE_ACTION_REQUEST_REGISTER))
      }
    })
  }

  // =========================================================================
  // Visibility Management
  // =========================================================================

  /**
   * Parse the enabled attribute into a Set of action IDs
   */
  private parseEnabledAttribute() {
    if (!this.enabled) {
      this.enabledSet = new Set()
    } else {
      this.enabledSet = new Set(
        this.enabled
          .split(',')
          .map((id) => id.trim())
          .filter((id) => id.length > 0)
      )
    }
    this.updateChildVisibility()
  }

  /**
   * Handle glide navigation end - read new enabled list from DOM
   */
  private handleGlideEnd = () => {
    // Look for the visibility data element that was swapped in with the new page
    const dataEl = document.querySelector('[data-stencila-actions-enabled]')
    if (dataEl) {
      const newEnabled = dataEl.getAttribute('data-stencila-actions-enabled') ?? ''
      this.enabled = newEnabled
      this.parseEnabledAttribute()
    }
  }

  /**
   * Update visibility of child action components based on enabledSet
   * If enabledSet is empty, all actions are shown (no filtering)
   */
  private updateChildVisibility() {
    // Query all direct child action components
    const children = this.querySelectorAll(
      'stencila-site-review, stencila-site-upload, stencila-site-remote'
    )

    children.forEach((child) => {
      const actionId = child.getAttribute('data-site-action')
      if (!actionId) return

      // If enabledSet is empty, show all; otherwise check if action is in the set
      const isVisible = this.enabledSet.size === 0 || this.enabledSet.has(actionId)

      if (isVisible) {
        child.removeAttribute('data-action-hidden')
        ;(child as HTMLElement).style.display = ''
      } else {
        child.setAttribute('data-action-hidden', '')
        ;(child as HTMLElement).style.display = 'none'
      }
    })

    // Trigger re-render to update FAB buttons
    this.requestUpdate()
  }

  /**
   * React to enabled attribute changes
   */
  override updated(changedProperties: Map<string, unknown>) {
    if (changedProperties.has('enabled')) {
      this.parseEnabledAttribute()
    }
  }

  // =========================================================================
  // UI Event Handlers
  // =========================================================================

  /**
   * Toggle the speed dial expansion
   */
  private toggleExpand() {
    this.expanded = !this.expanded
  }

  /**
   * Handle keyboard shortcuts (Escape to close)
   */
  private handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Escape' && this.expanded) {
      this.expanded = false
      e.preventDefault()
    }
  }

  /**
   * Handle clicks outside to close expanded menu
   */
  private handleDocumentClick = (e: MouseEvent) => {
    if (!this.expanded) return

    const target = e.target as Element
    if (!target.closest('stencila-site-actions')) {
      this.expanded = false
    }
  }

  /**
   * Handle action button click
   */
  private handleActionClick(action: ActionRegistration) {
    action.openPanel()
    this.expanded = false
  }

  /**
   * Get total badge count across visible actions only
   */
  private get totalBadgeCount(): number {
    let total = 0
    for (const [actionId, count] of this.badges.entries()) {
      // Only count if action is visible (enabledSet is empty or contains this action)
      if (this.enabledSet.size === 0 || this.enabledSet.has(actionId)) {
        total += count
      }
    }
    return total
  }

  /**
   * Get visible actions (filtered by enabledSet)
   */
  private get visibleActions(): ActionRegistration[] {
    return Array.from(this.actions.values()).filter(
      (action) =>
        this.enabledSet.size === 0 || this.enabledSet.has(action.id)
    )
  }

  // =========================================================================
  // Render Methods
  // =========================================================================

  override render() {
    // If no actions registered, just render the slot (children render their own FABs)
    if (this.actions.size === 0) {
      return html`<slot></slot>`
    }

    // If no visible actions for this route, just render the hidden slot
    if (this.visibleActions.length === 0) {
      return html`<slot></slot>`
    }

    return html`
      <div class="actions-container" data-direction=${this.direction}>
        ${this.mode === 'collapsed' ? this.renderMainFab() : nothing}
        ${this.renderActionButtons()} ${this.renderSlot()}
      </div>
    `
  }

  /**
   * Render the main FAB (shown in collapsed mode)
   */
  private renderMainFab() {
    const totalCount = this.totalBadgeCount

    return html`
      <button
        class="actions-fab ${this.expanded ? 'expanded' : ''}"
        @click=${this.toggleExpand}
        aria-label=${this.expanded ? 'Close actions menu' : 'Open actions menu'}
        aria-expanded=${this.expanded}
      >
        <span class="fab-icon i-lucide:plus"></span>
        ${totalCount > 0
          ? html`<span class="fab-badge"
              >${totalCount > 99 ? '99+' : totalCount}</span
            >`
          : nothing}
      </button>
    `
  }

  /**
   * Render the action buttons (only visible actions)
   */
  private renderActionButtons() {
    const showButtons = this.mode === 'expanded' || this.expanded

    if (!showButtons) {
      return nothing
    }

    // Sort visible actions by order (lower = closer to FAB)
    const sortedActions = this.visibleActions.sort((a, b) => a.order - b.order)

    return html`
      <div class="actions-buttons">
        ${sortedActions.map((action) => this.renderActionButton(action))}
      </div>
    `
  }

  /**
   * Render an individual action button
   */
  private renderActionButton(action: ActionRegistration) {
    const badgeCount = this.badges.get(action.id) ?? 0

    return html`
      <button
        class="action-button"
        @click=${() => this.handleActionClick(action)}
        aria-label=${action.label}
      >
        <span class="action-icon ${action.icon}"></span>
        ${badgeCount > 0
          ? html`<span class="action-badge"
              >${badgeCount > 99 ? '99+' : badgeCount}</span
            >`
          : nothing}
        <span class="action-label">${action.label}</span>
      </button>
    `
  }

  /**
   * Render the slot for child components
   */
  private renderSlot() {
    return html`<slot></slot>`
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'stencila-site-actions': StencilaSiteActions
  }
}
