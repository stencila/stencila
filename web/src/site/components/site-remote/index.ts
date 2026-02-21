import { html, nothing } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { type BaseFooterState, SiteAction, isLocalhost } from '../site-action'

import type {
  PickerMessage,
  RemoteService,
  WatchDirection,
  WatchMode,
} from './types'
import {
  GOOGLE_PICKER_URL,
  MICROSOFT_PICKER_URL,
  PICKER_ORIGIN,
  PICKER_POLL_INTERVAL,
  REMOTE_SUBMIT_PATH,
  generateTargetPath,
  getServiceDisplayName,
} from './utils'

/**
 * Site remote component
 *
 * Enables users to connect Google Docs or Microsoft 365 documents to the repository
 * via GitHub PRs, with optional bi-directional sync.
 */
@customElement('stencila-site-remote')
export class StencilaSiteRemote extends SiteAction {
  // =========================================================================
  // Abstract Method Implementations
  // =========================================================================

  get actionId() {
    return 'remote'
  }

  get actionIcon() {
    return 'i-lucide:cloud-download'
  }

  get actionLabel() {
    return 'Connect'
  }

  get actionOrder() {
    return 2
  }

  get badgeCount() {
    // No badge for remote - single document per PR flow
    return 0
  }

  get isActionAllowed() {
    const config = this.authStatus?.remoteConfig
    return config?.enabled === true && config?.allowed === true
  }

  // =========================================================================
  // Remote-Specific Properties
  // =========================================================================

  /**
   * Target path for new files
   */
  @property({ type: String, attribute: 'target-path' })
  targetPath: string = ''

  /**
   * Default output format (without dot, e.g., "smd", "md", "html")
   */
  @property({ type: String, attribute: 'default-format' })
  defaultFormat: string = 'smd'

  // =========================================================================
  // Remote-Specific State
  // =========================================================================

  /** Currently waiting for picker selection */
  @state()
  private isWaitingForPicker: boolean = false

  /** Selected remote document URL */
  @state()
  private remoteUrl: string = ''

  /** Selected document title */
  @state()
  private documentTitle: string = ''

  /** Selected service (gdoc or m365) */
  @state()
  private service: RemoteService | null = null

  /** Target path for the file (includes extension which determines format) */
  @state()
  private targetFilePath: string = ''

  /** Sync mode (includes direction or 'none' for no sync) */
  @state()
  private syncMode: WatchMode = 'bi'

  /** Whether sync dropdown is open */
  @state()
  private showSyncDropdown: boolean = false

  /** Optional PR message */
  @state()
  private message: string = ''

  /** Submission state */
  @state()
  private isSubmitting: boolean = false

  /** Whether submission was successful (PR creation is async) */
  @state()
  private isSubmitted: boolean = false

  /** Picker window reference */
  private pickerWindow: Window | null = null

  /** Picker close polling interval */
  private pickerPollInterval: number | null = null

  // =========================================================================
  // Lifecycle
  // =========================================================================

  override connectedCallback() {
    super.connectedCallback()
    window.addEventListener('message', this.handlePickerMessage)
  }

  override disconnectedCallback() {
    window.removeEventListener('message', this.handlePickerMessage)
    this.stopPickerClosePolling()
    super.disconnectedCallback()
  }

  /**
   * Apply server config from auth status to local state
   */
  protected override onAuthStatusReceived(): void {
    const config = this.authStatus?.remoteConfig
    if (!config) return

    // Apply default sync direction from server
    if (config.defaultSyncDirection) {
      this.syncMode = config.defaultSyncDirection as WatchMode
    }
  }

  // =========================================================================
  // Abstract Method Implementations - Auth
  // =========================================================================

  protected override applyPreviewDefaults(): void {
    this.authStatus = {
      hasSiteAccess: true,
      user: { id: 'preview', name: 'Preview User', avatar: '' },
      github: {
        connected: true,
        username: 'preview-user',
        canPush: true,
        source: 'oauth',
      },
      remoteConfig: {
        enabled: true,
        allowed: true,
        targetPath: this.targetPath,
        defaultFormat: this.defaultFormat,
        defaultSyncDirection: 'bi',
      },
      repo: { isPrivate: false, appInstalled: true },
      authorship: { canAuthorAsSelf: true, willBeBotAuthored: false },
    }
    // Also apply the preview defaults to state
    this.onAuthStatusReceived()
  }

  protected override calculateFooterState(): BaseFooterState {
    if (this.isSubmitting) {
      return { type: 'submitting' }
    }

    if (this.isSubmitted) {
      // PR creation is async for remote connections - no PR info yet
      return { type: 'success' }
    }

    if (this.authLoading || !this.authStatus) {
      return { type: 'loading' }
    }

    if (!this.authStatus.remoteConfig?.enabled) {
      return { type: 'blocked', reason: 'Remote documents are disabled' }
    }

    if (!this.authStatus.hasSiteAccess) {
      return { type: 'needSiteAccess', signInUrl: this.signInUrl }
    }

    // If anonymous submissions not allowed, require GitHub connection
    if (!this.anon && !this.authStatus.github?.connected) {
      if (!this.authStatus.user) {
        return { type: 'needStencilaSignIn', signInUrl: this.signInUrl }
      }
      return { type: 'needGitHubConnect' }
    }

    return {
      type: 'canSubmit',
      authorDescription: this.getAuthorDescription(),
    }
  }

  // =========================================================================
  // Picker Handling
  // =========================================================================

  private openPicker(service: RemoteService) {
    const baseUrl =
      service === 'gdoc' ? GOOGLE_PICKER_URL : MICROSOFT_PICKER_URL

    // Pass origin for secure postMessage verification
    const url = new URL(baseUrl)
    url.searchParams.set('origin', window.location.origin)

    this.pickerWindow = window.open(
      url.toString(),
      'picker',
      'width=800,height=600'
    )
    this.isWaitingForPicker = true
    this.service = service

    // Poll for popup close (user cancelled)
    this.startPickerClosePolling()
  }

  private startPickerClosePolling() {
    this.pickerPollInterval = window.setInterval(() => {
      if (this.pickerWindow?.closed) {
        this.handlePickerClosed()
      }
    }, PICKER_POLL_INTERVAL)
  }

  private stopPickerClosePolling() {
    if (this.pickerPollInterval) {
      clearInterval(this.pickerPollInterval)
      this.pickerPollInterval = null
    }
  }

  private handlePickerClosed() {
    this.stopPickerClosePolling()
    if (this.isWaitingForPicker && !this.remoteUrl) {
      // User closed without selecting - return to initial state
      this.isWaitingForPicker = false
      this.service = null
    }
    this.pickerWindow = null
  }

  private handlePickerMessage = (event: MessageEvent) => {
    // Security: Verify origin
    if (event.origin !== PICKER_ORIGIN) return

    // Security: Verify source is our picker window (prevents spoofing)
    if (event.source !== this.pickerWindow) return

    const data = event.data as PickerMessage
    if (data.type === 'document-selected') {
      this.stopPickerClosePolling()
      this.isWaitingForPicker = false
      this.remoteUrl = data.url
      this.documentTitle = data.title
      this.service = data.service

      // Auto-generate target path from title using default format
      const targetDir =
        this.authStatus?.remoteConfig?.targetPath || this.targetPath
      const defaultFormat =
        this.authStatus?.remoteConfig?.defaultFormat || this.defaultFormat
      this.targetFilePath = generateTargetPath(
        data.title,
        defaultFormat,
        targetDir
      )

      this.pickerWindow?.close()
      this.pickerWindow = null
    }
  }

  private cancelPicker() {
    this.stopPickerClosePolling()
    this.isWaitingForPicker = false
    this.service = null
    this.pickerWindow?.close()
    this.pickerWindow = null
  }

  // =========================================================================
  // Path Handling
  // =========================================================================

  private handlePathChange(e: Event) {
    const input = e.target as HTMLInputElement
    this.targetFilePath = input.value
  }

  // =========================================================================
  // Submission
  // =========================================================================

  private async handleSubmit() {
    if (!this.remoteUrl || !this.targetFilePath) return

    this.isSubmitting = true
    this.isSubmitted = false

    // Map syncMode to watch and syncDirection for API
    const watch = this.syncMode !== 'none'
    const watchDirection = watch ? (this.syncMode as WatchDirection) : undefined

    const body = {
          remoteUrl: this.remoteUrl,
          service: this.service,
          targetPath: this.targetFilePath,
          watch,
          watchDirection,
          message: this.message || `Add ${this.documentTitle} from ${getServiceDisplayName(this.service!)}`,
          authorAsSelf: !this.authStatus?.authorship?.willBeBotAuthored,
        }

    if (isLocalhost()) {
      // Show the payload that would be submitted
      this.showPreviewMock({
        endpoint: REMOTE_SUBMIT_PATH,
        method: 'POST',
        body,
      })
      this.isSubmitting = false
      return
    }

    try {
      const response = await this.apiFetch(REMOTE_SUBMIT_PATH, {
        method: 'POST',
        body,
      })

      if (response.ok) {
        // PR creation is async - just mark as submitted
        // User will receive email notification when PR is ready
        await response.json() // consume response
        this.isSubmitted = true
      } else {
        const error = await response.json()
        this.errorMessage = error.message || 'Failed to add remote document'
      }
    } catch {
      this.errorMessage = 'Network error'
    } finally {
      this.isSubmitting = false
    }
  }

  /**
   * Reset state for another document.
   * Context-aware: preserves nothing, fully resets to initial state.
   */
  protected override resetForAnother() {
    this.remoteUrl = ''
    this.documentTitle = ''
    this.service = null
    this.targetFilePath = ''
    this.message = ''
    this.isSubmitted = false
    this.syncMode =
      (this.authStatus?.remoteConfig?.defaultSyncDirection as WatchMode) ?? 'bi'
    this.showSyncDropdown = false
  }

  // =========================================================================
  // Rendering
  // =========================================================================

  protected override renderPanelContent() {
    // Show success state (real or preview)
    if (this.isSubmitted || this.isPreviewSuccess) {
      return this.renderSuccessState()
    }

    // Show waiting for picker state
    if (this.isWaitingForPicker) {
      return this.renderWaitingState()
    }

    // Show configuration state if document selected
    if (this.remoteUrl) {
      return this.renderConfigureState()
    }

    // Show service selection
    return this.renderServiceSelection()
  }

  private renderServiceSelection() {
    return html`
      <div class="remote-service-selection site-action-intro">
        <h4>Connect a document</h4>
        <p>Link a cloud document to sync with this site.</p>
        <div class="remote-service-buttons">
          <button
            class="remote-service-btn"
            @click=${() => this.openPicker('gdoc')}
          >
            <span class="i-bi:google service-icon"></span>
            <span class="service-name">Google Docs</span>
          </button>
          <button
            class="remote-service-btn"
            @click=${() => this.openPicker('m365')}
          >
            <span class="i-bi:microsoft service-icon"></span>
            <span class="service-name">Microsoft 365</span>
          </button>
        </div>
      </div>
    `
  }

  private renderWaitingState() {
    const serviceName = this.service
      ? getServiceDisplayName(this.service)
      : 'cloud service'
    return html`
      <div class="remote-waiting">
        <div class="remote-waiting-icon">
          <span class="i-lucide:loader-2 spinning"></span>
        </div>
        <p class="remote-waiting-title">Waiting for document selection...</p>
        <p class="remote-waiting-hint">
          A popup window has opened. Select a document from ${serviceName} to
          continue.
        </p>
        <button class="site-action-btn" @click=${this.cancelPicker}>
          Cancel
        </button>
      </div>
    `
  }

  private renderConfigureState() {
    return html`
      <div class="remote-configure">
        <!-- Two-card sync layout: remote on top, local on bottom -->
        ${this.renderRemoteDocCard()}
        ${this.renderSyncBadge()}
        ${this.renderLocalFileCard()}

        <!-- Footer -->
        ${this.renderPanelFooter({
          disabled: !this.remoteUrl || !this.targetFilePath || this.isSubmitting,
          onSubmit: () => this.handleSubmit(),
        })}
      </div>
    `
  }

  /**
   * Render local file card (top card - GitHub repo file)
   */
  private renderLocalFileCard() {
    return html`
      <div class="sync-card local-card">
        <span class="i-bi:github card-icon"></span>
        <input
          type="text"
          class="file-path-input"
          .value=${this.targetFilePath}
          @input=${this.handlePathChange}
        />
      </div>
    `
  }

  /**
   * Render sync direction badge with dropdown
   */
  private renderSyncBadge() {
    const labels: Record<WatchMode, { arrows: string; text: string; className: string }> = {
      'bi': { arrows: '↑↓', text: 'BI-DIRECTIONAL', className: 'bi' },
      'from-remote': { arrows: '↓', text: 'FROM REMOTE', className: 'from' },
      'to-remote': { arrows: '↑', text: 'TO REMOTE', className: 'to' },
      'none': { arrows: '—', text: 'NO SYNC', className: 'none' },
    }
    const current = labels[this.syncMode]

    return html`
      <div class="sync-badge-wrapper">
        <button
          class="sync-badge ${current.className}"
          @click=${this.toggleSyncDropdown}
          aria-expanded=${this.showSyncDropdown}
          aria-haspopup="listbox"
        >
          <span class="sync-arrows">${current.arrows}</span>
          <span class="sync-label">${current.text}</span>
          <span class="i-lucide:chevron-down caret"></span>
        </button>
        ${this.showSyncDropdown ? this.renderSyncDropdown(labels) : nothing}
      </div>
    `
  }

  /**
   * Render sync direction dropdown options
   */
  private renderSyncDropdown(
    labels: Record<WatchMode, { arrows: string; text: string; className: string }>
  ) {
    const options: WatchMode[] = ['bi', 'from-remote', 'to-remote', 'none']

    return html`
      <div class="sync-dropdown" role="listbox">
        ${options.map((mode) => {
          const { arrows, text, className } = labels[mode]
          const isSelected = this.syncMode === mode
          return html`
            <button
              class="sync-dropdown-item ${className} ${isSelected ? 'selected' : ''}"
              role="option"
              aria-selected=${isSelected}
              @click=${() => this.selectSyncMode(mode)}
            >
              <span class="sync-arrows">${arrows}</span>
              <span class="sync-label">${text}</span>
            </button>
          `
        })}
      </div>
    `
  }

  /**
   * Toggle sync dropdown visibility
   */
  private toggleSyncDropdown() {
    this.showSyncDropdown = !this.showSyncDropdown
  }

  /**
   * Select a sync mode and close dropdown
   */
  private selectSyncMode(mode: WatchMode) {
    this.syncMode = mode
    this.showSyncDropdown = false
  }

  /**
   * Render remote document card (bottom card - cloud document)
   */
  private renderRemoteDocCard() {
    const icon = this.service === 'gdoc'
      ? 'i-bi:google'
      : 'i-bi:microsoft'
    const serviceName = this.service
      ? getServiceDisplayName(this.service)
      : 'Remote'

    return html`
      <div class="sync-card remote-card">
        <span class="${icon} card-icon"></span>
        <div class="doc-info">
          <span class="doc-title">${this.documentTitle || 'Untitled'}</span>
          <span class="doc-service">${serviceName}</span>
        </div>
        <button
          class="change-btn"
          @click=${() => this.service && this.openPicker(this.service)}
        >
          Change
        </button>
      </div>
    `
  }

  private renderSuccessState() {
    const isPreview = this.isPreviewSuccess

    return this.renderSuccessStateBase({
      title: 'Request submitted',
      hintText:
        "Your connection request is being processed. You'll receive an email when it's ready for review.",
      addAnotherLabel: 'Connect More',
      isPreview,
      icon: 'i-lucide:clock',
      // No prNumber/prUrl - PR creation is async
    })
  }
}
