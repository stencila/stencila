import { LitElement, html, nothing, type TemplateResult } from 'lit'
import { property, state } from 'lit/decorators.js'

import {
  type SiteAuthStatusResponse,
  type BaseFooterState,
  type ActionRegistration,
  type ActionPosition,
  SITE_ACTION_REGISTER,
  SITE_ACTION_BADGE_UPDATE,
  SITE_ACTION_UNREGISTER,
  SITE_ACTION_REQUEST_REGISTER,
} from './types'
import {
  isLocalhost,
  isDevMode,
  GITHUB_OAUTH_URL,
  getCachedAuthStatus,
  invalidateAuthCache,
} from './utils'

// Re-export types and utils for consumers
export * from './types'
export * from './utils'

/**
 * Abstract base class for site action components (review, upload, subscribe, connect, etc.)
 *
 * Provides:
 * - Common properties (workspaceId, position, public, anon, hideFab)
 * - Authentication handling (Clerk token, API fetch, auth status)
 * - Panel state management (open/close)
 * - Self-registration with parent site-actions container
 * - Badge count updates
 * - Common render methods for FAB and panel structure
 *
 * Subclasses must implement:
 * - actionId, actionIcon, actionLabel getters
 * - authEndpoint getter
 * - badgeCount getter
 * - applyDevDefaults() method
 * - calculateFooterState() method
 * - renderPanelContent() method
 */
export abstract class SiteAction extends LitElement {
  // =========================================================================
  // Common Properties
  // =========================================================================

  /**
   * Workspace ID from site configuration
   */
  @property({ type: String, attribute: 'workspace-id' })
  workspaceId: string = ''

  /**
   * Position of the action affordance on the page
   */
  @property({ type: String })
  position: ActionPosition = 'bottom-right'

  /**
   * Whether public (non-team) submissions are allowed
   */
  @property({ type: Boolean })
  public: boolean = true

  /**
   * Whether anonymous (no GitHub auth) submissions are allowed
   */
  @property({ type: Boolean })
  anon: boolean = false

  /**
   * Hide the FAB when inside a site-actions container
   */
  @property({ type: Boolean, attribute: 'hide-fab' })
  hideFab: boolean = false

  // =========================================================================
  // Common State
  // =========================================================================

  /**
   * Whether the action panel is open
   */
  @state()
  protected isOpen: boolean = false

  /**
   * Auth status from the API
   */
  @state()
  protected authStatus: SiteAuthStatusResponse | null = null

  /**
   * Whether auth status is being loaded
   */
  @state()
  protected authLoading: boolean = true

  /**
   * Error message to display
   */
  @state()
  protected errorMessage: string = ''

  /**
   * Last badge count dispatched (for dedup)
   */
  private lastBadgeCount: number = -1

  // =========================================================================
  // Abstract Methods - Must be implemented by subclasses
  // =========================================================================

  /**
   * Unique identifier for this action (e.g., 'review', 'upload')
   */
  abstract get actionId(): string

  /**
   * Icon class for this action (e.g., 'i-lucide:message-square-plus')
   */
  abstract get actionIcon(): string

  /**
   * Display label for this action (e.g., 'Review', 'Upload')
   */
  abstract get actionLabel(): string

  /**
   * API endpoint for auth status.
   */
  get authEndpoint(): string {
    return '/__stencila/auth/status'
  }

  /**
   * Current badge count to display on FAB
   */
  abstract get badgeCount(): number

  /**
   * Whether the current user is allowed to use this action.
   * Returns false if the action is disabled or the user lacks permission.
   * Used to hide the FAB when the user cannot submit.
   */
  abstract get isActionAllowed(): boolean

  /**
   * Apply development defaults when on localhost without API
   */
  protected abstract applyDevDefaults(): void

  /**
   * Optional hook called after auth status is successfully received.
   * Subclasses can override to apply action-specific config from the response.
   */
  protected onAuthStatusReceived(): void {
    // Default: no-op. Subclasses can override.
  }

  /**
   * Calculate the current footer state based on auth and submission status
   */
  protected abstract calculateFooterState(): BaseFooterState

  /**
   * Render the action-specific panel content
   */
  protected abstract renderPanelContent(): TemplateResult | typeof nothing

  // =========================================================================
  // Lifecycle
  // =========================================================================

  /**
   * Use Light DOM so theme CSS can style the component
   */
  protected override createRenderRoot() {
    return this
  }

  override connectedCallback() {
    super.connectedCallback()

    // Listen for request-register event (fallback from parent)
    this.addEventListener(
      SITE_ACTION_REQUEST_REGISTER,
      this.handleRequestRegister
    )

    // Fetch auth status
    this.fetchAuthStatus()
  }

  override disconnectedCallback() {
    // Remove request-register listener
    this.removeEventListener(
      SITE_ACTION_REQUEST_REGISTER,
      this.handleRequestRegister
    )

    // Dispatch unregister event
    this.dispatchEvent(
      new CustomEvent(SITE_ACTION_UNREGISTER, {
        bubbles: true,
        composed: true,
        detail: { id: this.actionId },
      })
    )

    super.disconnectedCallback()
  }

  protected override firstUpdated() {
    // Dispatch registration event to parent site-actions
    this.dispatchRegistration()
  }

  protected override updated() {
    // Dispatch badge update if count changed
    const currentCount = this.badgeCount
    if (currentCount !== this.lastBadgeCount) {
      this.lastBadgeCount = currentCount
      this.dispatchEvent(
        new CustomEvent(SITE_ACTION_BADGE_UPDATE, {
          bubbles: true,
          composed: true,
          detail: { id: this.actionId, count: currentCount },
        })
      )
    }
  }

  // =========================================================================
  // Event Handling
  // =========================================================================

  /**
   * Handle request to re-register (fallback from parent scan)
   */
  private handleRequestRegister = () => {
    this.dispatchRegistration()
  }

  /**
   * Dispatch registration event to parent
   */
  private dispatchRegistration() {
    const registration: ActionRegistration = {
      id: this.actionId,
      icon: this.actionIcon,
      label: this.actionLabel,
      openPanel: () => this.openPanel(),
    }

    this.dispatchEvent(
      new CustomEvent(SITE_ACTION_REGISTER, {
        bubbles: true,
        composed: true,
        detail: registration,
      })
    )

    // Set data attribute for parent fallback scan
    this.setAttribute('data-site-action', this.actionId)
  }

  // =========================================================================
  // Panel State
  // =========================================================================

  /**
   * Open the action panel (public method for parent components)
   */
  openPanel() {
    this.isOpen = true
  }

  /**
   * Close the action panel
   */
  closePanel() {
    this.isOpen = false
  }

  /**
   * Toggle the action panel
   */
  protected togglePanel(event?: Event) {
    event?.stopPropagation()
    this.isOpen = !this.isOpen
  }

  // =========================================================================
  // API & Authentication
  // =========================================================================

  /**
   * Get the API base URL for endpoints.
   * On localhost, uses the workspace's stencila.site domain.
   * On production (*.stencila.site), uses same-origin.
   */
  protected get apiBase(): string {
    if (isLocalhost() && this.workspaceId) {
      return `https://${this.workspaceId}.stencila.site`
    }
    return ''
  }

  /**
   * Get headers for API requests, including Clerk token if available
   */
  protected async getAuthHeaders(): Promise<Record<string, string>> {
    // Try to get Clerk token if Clerk SDK is available
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const clerk = (window as any).Clerk
    if (clerk?.session) {
      try {
        const token = await clerk.session.getToken()
        if (token) {
          return { Authorization: `Bearer ${token}` }
        }
      } catch (e) {
        console.warn(`[Site${this.actionLabel}] Failed to get Clerk token:`, e)
      }
    }
    return {}
  }

  /**
   * Unified API fetch helper with auth headers and credentials
   */
  protected async apiFetch(
    path: string,
    options: { method?: string; body?: unknown } = {}
  ): Promise<Response> {
    const authHeaders = await this.getAuthHeaders()
    const headers: Record<string, string> = { ...authHeaders }
    if (options.body) {
      headers['Content-Type'] = 'application/json'
    }

    return fetch(this.apiBase + path, {
      method: options.method ?? 'GET',
      headers,
      credentials: isLocalhost() ? 'include' : 'same-origin',
      ...(options.body && { body: JSON.stringify(options.body) }),
    })
  }

  /**
   * Fetch auth status from the API.
   *
   * Uses a shared cache so multiple action components on the same page
   * don't make duplicate requests.
   *
   * @param forceRefresh - If true, bypass cache and fetch fresh data
   */
  protected async fetchAuthStatus(forceRefresh: boolean = false) {
    this.authLoading = true

    // In dev mode on localhost, skip API and use permissive defaults
    if (isDevMode(this.actionId)) {
      console.log(`[Site${this.actionLabel}] Dev mode enabled, skipping API`)
      this.applyDevDefaults()
      this.authLoading = false
      return
    }

    console.log(
      `[Site${this.actionLabel}] Fetching auth status from:`,
      this.apiBase + this.authEndpoint,
      forceRefresh ? '(forced refresh)' : '(may use cache)'
    )

    try {
      // Use cached auth status (shared across all action components)
      const result = await getCachedAuthStatus(
        this.apiBase,
        this.authEndpoint,
        () => this.getAuthHeaders(),
        forceRefresh
      )

      if (result) {
        this.authStatus = result
        console.log(`[Site${this.actionLabel}] Auth status:`, this.authStatus)
        this.onAuthStatusReceived()
      } else {
        console.error(`[Site${this.actionLabel}] Failed to fetch auth status`)
        this.applyDevDefaults()
      }
    } catch (e) {
      console.error(`[Site${this.actionLabel}] Auth status fetch failed:`, e)
      this.applyDevDefaults()
    } finally {
      this.authLoading = false
    }
  }

  /**
   * Refresh auth status (e.g., after returning from sign-in).
   * Invalidates the cache and fetches fresh data.
   */
  protected refreshAuthStatus() {
    invalidateAuthCache(this.apiBase)
    this.fetchAuthStatus(true)
  }

  // =========================================================================
  // Common URL Builders
  // =========================================================================

  /**
   * Get the sign-in URL with workspace ID and return URL
   */
  protected get signInUrl(): string {
    const url = new URL(
      `https://${this.workspaceId}.stencila.site/__stencila/auth/signin`
    )
    url.searchParams.set('return', window.location.href)
    return url.toString()
  }

  /**
   * Get the GitHub connect URL with workspace ID and return URL
   */
  protected get gitHubConnectUrl(): string {
    const url = new URL(GITHUB_OAUTH_URL)
    url.searchParams.set('workspace_id', this.workspaceId)
    url.searchParams.set('return_url', window.location.href)
    return url.toString()
  }

  // =========================================================================
  // Common Position Helpers
  // =========================================================================

  /**
   * Get CSS styles for FAB positioning
   */
  protected get fabPositionStyles(): string {
    const offset = '16px'
    switch (this.position) {
      case 'top-right':
        return `top: ${offset}; right: ${offset};`
      case 'top-left':
        return `top: ${offset}; left: ${offset};`
      case 'bottom-left':
        return `bottom: ${offset}; left: ${offset};`
      case 'bottom-right':
      default:
        return `bottom: ${offset}; right: ${offset};`
    }
  }

  /**
   * Get CSS styles for panel positioning
   */
  protected get panelPositionStyles(): string {
    const edgeOffset = '16px'
    const fabEdgeOffset = 16
    const fabSize = 48
    const panelOffset = `${fabEdgeOffset + fabSize / 2}px`
    switch (this.position) {
      case 'top-right':
        return `top: ${panelOffset}; right: ${edgeOffset};`
      case 'top-left':
        return `top: ${panelOffset}; left: ${edgeOffset};`
      case 'bottom-left':
        return `bottom: ${panelOffset}; left: ${edgeOffset};`
      case 'bottom-right':
      default:
        return `bottom: ${panelOffset}; right: ${edgeOffset};`
    }
  }

  // =========================================================================
  // Common Render Helpers
  // =========================================================================

  /**
   * Render the FAB (Floating Action Button)
   * Hidden when inside a site-actions container (hideFab) or when user is not allowed
   */
  protected renderFab() {
    // Hide FAB if inside container or user not allowed (after auth loaded)
    if (this.hideFab || (!this.authLoading && !this.isActionAllowed)) {
      return nothing
    }

    const count = this.badgeCount

    return html`
      <button
        class="site-action-fab"
        style=${this.fabPositionStyles}
        @click=${this.togglePanel}
        aria-label=${count > 0
          ? `Open ${this.actionLabel.toLowerCase()} panel (${count} pending)`
          : `Open ${this.actionLabel.toLowerCase()} panel`}
        aria-expanded=${this.isOpen}
      >
        <span class="${this.actionIcon} fab-icon"></span>
        ${count > 0
          ? html`<span class="site-action-badge">${count > 99 ? '99+' : count}</span>`
          : nothing}
      </button>
    `
  }

  /**
   * Render the panel header
   */
  protected renderPanelHeader() {
    return html`
      <div class="site-action-panel-header">
        <span class="panel-title">${this.actionLabel}</span>
        <button
          class="panel-close"
          @click=${() => this.closePanel()}
          aria-label="Close panel"
        >
          <span class="i-lucide:x"></span>
        </button>
      </div>
    `
  }

  /**
   * Render the panel footer status based on footer state
   */
  protected renderFooterStatus() {
    const state = this.calculateFooterState()

    switch (state.type) {
      case 'loading':
        return html`<p class="footer-status muted">Checking access...</p>`

      case 'submitting':
        return html`<p class="footer-status muted">Creating pull request...</p>`

      case 'success':
        return html`<p class="footer-status success">
          PR #${state.prNumber} created
          <a
            href=${state.prUrl}
            target="_blank"
            rel="noopener noreferrer"
            class="pr-link"
            >View</a
          >
        </p>`

      case 'error':
        return html`<p class="footer-status error">${state.message}</p>`

      case 'blocked':
        return html`<p class="footer-status warning">
          <span class="i-lucide:alert-triangle warning-icon"></span>
          ${state.reason}
        </p>`

      case 'needSiteAccess':
        return html`<p class="footer-status">
          <a href=${state.signInUrl} class="auth-link">Sign in to site to submit</a>
        </p>`

      case 'needStencilaSignIn':
        return html`<p class="footer-status">
          <a href=${state.signInUrl} class="auth-link">Sign in to Stencila</a>
          (required for private repos)
        </p>`

      case 'needGitHubConnect':
        return html`<p class="footer-status">
          <a
            href=${this.gitHubConnectUrl}
            target="_blank"
            rel="noopener noreferrer"
            class="auth-link"
            >Connect GitHub</a
          >
          to submit
        </p>`

      case 'canSubmit':
        return html`<p class="footer-status muted">${state.authorDescription}</p>`

      default:
        return nothing
    }
  }

  /**
   * Render the complete panel structure
   */
  protected renderPanel() {
    if (!this.isOpen) {
      return nothing
    }

    return html`
      <div class="site-action-panel" style=${this.panelPositionStyles}>
        ${this.renderPanelHeader()}
        <div class="site-action-panel-content">
          ${this.renderPanelContent()}
        </div>
      </div>
    `
  }

  /**
   * Render error modal
   */
  protected renderErrorModal() {
    if (!this.errorMessage) {
      return nothing
    }

    return html`
      <div class="site-action-backdrop" @click=${() => (this.errorMessage = '')}></div>
      <div class="site-action-modal error">
        <h4>${this.actionLabel} Error</h4>
        <p>${this.errorMessage}</p>
        <div class="buttons">
          <button class="site-action-btn primary" @click=${() => (this.errorMessage = '')}>OK</button>
        </div>
      </div>
    `
  }
}
