import { html, nothing } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { GlideEvents } from '../../glide/events'
import { navigate } from '../../glide/glide'
import type { GlideEventDetail } from '../../glide/types'
import {
  SiteAction,
  type BaseFooterState,
  isLocalhost,
  getPathname,
  isStencilaHosted,
} from '../site-action'

import {
  supportsHighlightAPI,
  findTextPosition,
  findNodeWithId,
  getCharOffset,
  createRangeForItem,
  caretRangeFromPoint,
  rangeContainsPoint,
} from './dom'
import type {
  ItemAddDetail,
  ItemClickDetail,
  ItemDeleteDetail,
  ItemEditDetail,
  SelectionInfo,
} from './item'
import type {
  ReviewItem,
  ReviewItemAnchor,
  SourceInfo,
  ReviewResponse,
  ApiError,
} from './types'
import {
  SHARE_PARAM,
  encodeReviewForUrl,
  extractSharedReview,
  hasSharedReview,
} from './url'
import {
  STORAGE_KEY_ITEMS,
  STORAGE_KEY_SOURCE,
  REVIEW_SUBMIT_PATH,
} from './utils'
import './item'

/**
 * Site review component
 *
 * Enables users to select text on rendered pages and submit comments/suggestions.
 * Supports multi-block selections and batches multiple annotations into a single review.
 * Persists review items in localStorage across page refreshes.
 */
@customElement('stencila-site-review')
export class StencilaSiteReview extends SiteAction {
  // =========================================================================
  // Abstract Method Implementations
  // =========================================================================

  get actionId() {
    return 'review'
  }

  get actionIcon() {
    return 'i-lucide:message-square-plus'
  }

  get actionLabel() {
    return 'Review'
  }

  get badgeCount() {
    return this.pendingItems.length
  }

  get isActionAllowed() {
    const config = this.authStatus?.reviewConfig
    return config?.enabled === true && config?.allowed === true
  }

  // =========================================================================
  // Review-Specific Properties
  // =========================================================================

  /**
   * Allowed review types (comma-separated: "comment", "suggestion", or "comment,suggestion")
   */
  @property({ type: String })
  types: string = 'comment,suggestion'

  /**
   * Minimum characters required to trigger the affordance
   */
  @property({ type: Number, attribute: 'min-selection' })
  minSelection: number = 1

  /**
   * Maximum characters allowed in a selection
   */
  @property({ type: Number, attribute: 'max-selection' })
  maxSelection: number = 5000

  /**
   * Enable keyboard shortcuts
   */
  @property({ type: Boolean })
  shortcuts: boolean = false

  /**
   * Check if comments are allowed based on types attribute
   */
  private get allowsComments(): boolean {
    return this.types.includes('comment')
  }

  /**
   * Check if suggestions are allowed based on types attribute
   */
  private get allowsSuggestions(): boolean {
    return this.types.includes('suggestion')
  }

  // =========================================================================
  // Review-Specific State
  // =========================================================================

  /**
   * Current selection anchors
   */
  @state()
  private currentSelection: {
    start: ReviewItemAnchor
    end: ReviewItemAnchor
    selectedText: string
    rect: DOMRect
  } | null = null

  /**
   * Current media element selection (for comments on images, audio, video)
   */
  @state()
  private currentMediaSelection: {
    nodeId: string
    element: Element
    rect: DOMRect
    mediaType: string
  } | null = null

  /**
   * Pending review items (persisted to localStorage)
   */
  @state()
  private pendingItems: ReviewItem[] = []

  /**
   * Whether the input modal is shown
   */
  @state()
  private showInput: boolean = false

  /**
   * Current input type (comment or suggestion)
   */
  @state()
  private inputType: 'comment' | 'suggestion' = 'comment'

  /**
   * Source info for the review (set when first item is added)
   */
  @state()
  private sourceInfo: SourceInfo | null = null

  /**
   * Whether to show the commit mismatch modal (production only)
   */
  @state()
  private showCommitMismatch: boolean = false

  /**
   * Whether a review submission is in progress
   */
  @state()
  private submitting: boolean = false

  /**
   * Result from successful review submission
   */
  @state()
  private submitResult: ReviewResponse | null = null

  /**
   * Error message from failed submission
   */
  @state()
  private submitError: string = ''

  /**
   * Whether the FAB is pulsing (after item added)
   */
  @state()
  private isFabPulsing: boolean = false

  /**
   * Currently selected/active item index in the panel
   */
  @state()
  private activeItemIndex: number | null = null

  /**
   * Set of expanded page group paths (for collapsible sections)
   * Current page is always expanded by default
   */
  @state()
  private expandedPageGroups: Set<string> = new Set()

  /**
   * Tooltip message to show near share button
   */
  @state()
  private shareTooltip: string = ''

  /**
   * Item index to activate after cross-page navigation completes
   */
  private pendingActivation: number | null = null

  /**
   * Tracks highlight ranges and their associated item indices
   * Used for click detection on highlights
   */
  private highlightRanges: Array<{
    range: Range;
    itemIndex: number;
    type: 'comment' | 'suggestion';
  }> = []

  /**
   * CSS Highlight objects for the Custom Highlight API
   */
  private commentHighlight: Highlight | null = null
  private suggestionHighlight: Highlight | null = null
  private activeHighlight: Highlight | null = null
  private inputHighlight: Highlight | null = null

  // =========================================================================
  // Lifecycle
  // =========================================================================

  override connectedCallback() {
    super.connectedCallback()
    this.loadFromStorage()

    // Check for shared review in URL parameter
    if (hasSharedReview()) {
      this.loadSharedReviewFromUrl()
    }

    document.addEventListener('selectionchange', this.handleSelectionChange)
    document.addEventListener('mouseup', this.handleMouseUp)
    document.addEventListener('visibilitychange', this.handleVisibilityChange)
    document.addEventListener('keydown', this.handleKeyDown)
    window.addEventListener('scroll', this.handleScroll, { passive: true })
    window.addEventListener(GlideEvents.END, this.handleGlideEnd)

    // Apply highlights after initial load (with small delay for DOM to be ready)
    // Also auto-expand the current page group
    requestAnimationFrame(() => {
      this.applyHighlights()
      this.expandCurrentPageGroup()
    })
  }

  override disconnectedCallback() {
    super.disconnectedCallback()
    document.removeEventListener('selectionchange', this.handleSelectionChange)
    document.removeEventListener('mouseup', this.handleMouseUp)
    document.removeEventListener(
      'visibilitychange',
      this.handleVisibilityChange,
    )
    document.removeEventListener('keydown', this.handleKeyDown)
    window.removeEventListener('scroll', this.handleScroll)
    window.removeEventListener(GlideEvents.END, this.handleGlideEnd)

    // Clear highlights when component is removed
    this.clearHighlights()
  }

  /**
   * Handle keyboard shortcuts
   * - Ctrl+Shift+C: Add comment on current selection
   * - Ctrl+Shift+S: Add suggestion on current selection
   * - Escape: Cancel current input / close panel
   */
  private handleKeyDown = (e: KeyboardEvent) => {
    // Handle Escape to cancel (in priority order)
    if (e.key === 'Escape') {
      if (this.showInput) {
        this.handleCancel()
        e.preventDefault()
      } else if (this.isOpen) {
        this.isOpen = false
        e.preventDefault()
      }
      return
    }

    // Only process shortcuts if enabled
    if (!this.shortcuts) {
      return
    }

    // Check for Ctrl+Shift+C (comment) or Ctrl+Shift+S (suggestion)
    if (e.ctrlKey && e.shiftKey) {
      if (e.key === 'C' && this.allowsComments) {
        // If text is selected, comment on selection; otherwise page-level comment
        if (this.currentSelection) {
          this.handleComment()
        } else {
          this.handlePageComment()
        }
        e.preventDefault()
      } else if (
        e.key === 'S' &&
        this.currentSelection &&
        this.allowsSuggestions
      ) {
        this.handleSuggest()
        e.preventDefault()
      }
    }
  }

  /**
   * Handle visibility change to refresh auth status when returning from sign-in
   */
  private handleVisibilityChange = () => {
    if (document.visibilityState === 'visible') {
      this.refreshAuthStatus()
    }
  }

  // =========================================================================
  // Abstract Method Implementations
  // =========================================================================

  /**
   * Apply permissive defaults for localhost preview.
   */
  protected override applyPreviewDefaults() {
    this.authStatus = {
      hasSiteAccess: true,
      reviewConfig: {
        enabled: true,
        allowed: true,
        allowPublic: true,
        allowAnonymous: true,
      },
      repo: {
        isPrivate: false,
        appInstalled: true,
      },
      authorship: {
        canAuthorAsSelf: false,
        willBeBotAuthored: true,
        reason: 'Preview mode - no GitHub connected',
      },
    }
  }

  /**
   * Calculate the current footer state based on auth and submission status.
   * Evaluated in priority order - first matching state wins.
   */
  protected override calculateFooterState(): BaseFooterState {
    // 1. Loading
    if (this.authLoading) {
      return { type: 'loading' }
    }

    // 2. Submitting
    if (this.submitting) {
      return { type: 'submitting' }
    }

    // 3. Success (persists until next action)
    if (this.submitResult) {
      return {
        type: 'success',
        prNumber: this.submitResult.prNumber,
        prUrl: this.submitResult.prUrl,
      }
    }

    // 4. Error (clears on next action)
    if (this.submitError) {
      return { type: 'error', message: this.submitError }
    }

    // 5. Check if reviews are enabled (guards against missing/disabled reviewConfig)
    if (!this.authStatus?.reviewConfig?.enabled) {
      return { type: 'blocked', reason: 'Reviews are disabled' }
    }

    // 6. Blocked: private repo without app
    if (this.isBlockedPrivateRepo) {
      return { type: 'blocked', reason: this.blockedReason ?? '' }
    }

    // 7. Need site access (site requires auth to submit)
    if (!this.authStatus.reviewConfig.allowPublic && !this.authStatus.hasSiteAccess) {
      return { type: 'needSiteAccess', signInUrl: this.signInUrl }
    }

    // 8. Need Stencila sign-in (private repo - OAuth lacks scope)
    if (this.requiresStencilaSignIn) {
      return { type: 'needStencilaSignIn', signInUrl: this.signInUrl }
    }

    // 9. Need GitHub connect (public allows reviews but requires attribution)
    if (this.showGitHubConnect) {
      return { type: 'needGitHubConnect' }
    }

    // 10. Can submit
    return { type: 'canSubmit', authorDescription: this.prAuthorDescription }
  }

  /**
   * Render the review-specific panel content.
   */
  protected override renderPanelContent() {
    const itemCount = this.pendingItems.length

    if (itemCount === 0) {
      return html`
        <div class="site-action-empty-state">
          <h4>Ready for your feedback</h4>
          <p>Select text on the page to comment or suggest a change.</p>
          <button
            class="site-action-btn site-action-btn-secondary add-page-comment"
            @click=${(e: Event) => {
              e.stopPropagation()
              this.handlePageComment()
            }}
          >
            <span class="i-lucide:message-circle site-action-btn-icon"></span>
            Add page comment
          </button>
        </div>
      `
    }

    return html` ${this.renderPanelItems()} ${this.renderPanelFooter()} `
  }

  /**
   * Load pending items and source info from localStorage
   */
  private loadFromStorage() {
    try {
      const storedItems = localStorage.getItem(STORAGE_KEY_ITEMS)
      if (storedItems) {
        this.pendingItems = JSON.parse(storedItems)
        console.log(
          '[SiteReview] Loaded items from storage:',
          this.pendingItems.length,
        )
      }
      const storedSource = localStorage.getItem(STORAGE_KEY_SOURCE)
      if (storedSource) {
        this.sourceInfo = JSON.parse(storedSource)
        console.log(
          '[SiteReview] Loaded source info from storage:',
          this.sourceInfo,
        )
      }
    } catch (e) {
      console.error('[SiteReview] Failed to load from storage:', e)
    }
  }

  /**
   * Save pending items and source info to localStorage
   */
  private saveToStorage() {
    try {
      localStorage.setItem(
        STORAGE_KEY_ITEMS,
        JSON.stringify(this.pendingItems),
      )
      if (this.sourceInfo) {
        localStorage.setItem(
          STORAGE_KEY_SOURCE,
          JSON.stringify(this.sourceInfo),
        )
      } else {
        localStorage.removeItem(STORAGE_KEY_SOURCE)
      }
      console.log(
        '[SiteReview] Saved to storage:',
        this.pendingItems.length,
        'items',
      )
    } catch (e) {
      console.error('[SiteReview] Failed to save to storage:', e)
    }
  }

  // =========================================================================
  // URL Sharing Methods
  // =========================================================================

  /**
   * Load shared review from URL parameter
   * Merges with existing items and cleans the URL
   */
  private async loadSharedReviewFromUrl(): Promise<void> {
    const { data, cleanUrl } = await extractSharedReview()

    if (data) {
      // Merge with existing items (dedupe by path+nodeId+offset)
      this.mergeSharedItems(data.items)

      if (data.source && !this.sourceInfo) {
        this.sourceInfo = data.source
      }

      this.saveToStorage()
      this.isOpen = true // Auto-open panel

      // Apply highlights after a short delay for DOM to be ready
      requestAnimationFrame(() => {
        this.applyHighlights()
        this.expandCurrentPageGroup()
      })

      console.log(
        '[SiteReview] Loaded shared review:',
        data.items.length,
        'items',
      )
    }

    // Clean URL regardless of success (remove the parameter)
    history.replaceState({}, '', cleanUrl)
  }

  /**
   * Merge shared items with existing pending items
   * Deduplicates by path + start.nodeId + start.offset
   */
  private mergeSharedItems(newItems: ReviewItem[]): void {
    const isDuplicate = (item: ReviewItem) =>
      this.pendingItems.some(
        (existing) =>
          existing.path === item.path &&
          existing.start.nodeId === item.start.nodeId &&
          existing.start.offset === item.start.offset,
      )

    const uniqueItems = newItems.filter((item) => !isDuplicate(item))
    this.pendingItems = [...this.pendingItems, ...uniqueItems]
  }

  /**
   * Generate share URL, copy to clipboard, and show tooltip
   */
  private async handleShare(): Promise<void> {
    const result = await encodeReviewForUrl(
      this.pendingItems,
      this.sourceInfo ?? undefined,
    )

    const url = new URL(window.location.href)
    url.searchParams.set(SHARE_PARAM, result.encoded)

    try {
      await navigator.clipboard.writeText(url.toString())

      // Show appropriate tooltip message
      if (result.truncated) {
        this.showShareTooltip(
          `Copied ${result.includedCount} of ${this.pendingItems.length}`,
        )
      } else {
        this.showShareTooltip('Copied!')
      }

      console.log('[SiteReview] Share URL copied to clipboard')
    } catch (e) {
      console.error('[SiteReview] Failed to copy share URL:', e)
      this.showShareTooltip('Failed to copy')
    }
  }

  /**
   * Show tooltip near share button briefly
   */
  private showShareTooltip(message: string, duration: number = 1500): void {
    this.shareTooltip = message
    setTimeout(() => {
      this.shareTooltip = ''
    }, duration)
  }

  // =========================================================================
  // Helper Methods for Auth State
  // =========================================================================

  /**
   * Check if the user can submit a review based on current auth state
   */
  private get canSubmitReview(): boolean {
    const config = this.authStatus?.reviewConfig
    if (!config?.enabled) return false

    // Must have site access if not public
    if (!config.allowPublic && !this.authStatus?.hasSiteAccess) return false

    // Must have GitHub if anonymous not allowed
    if (!config.allowAnonymous && !this.authStatus?.github?.connected) return false

    // Blocked: private repo, no push access, no app installed
    if (this.isBlockedPrivateRepo) return false

    return true
  }

  /**
   * Check if we're blocked due to private repo without app
   */
  private get isBlockedPrivateRepo(): boolean {
    const repo = this.authStatus?.repo
    const github = this.authStatus?.github

    if (!repo?.isPrivate || repo?.appInstalled) {
      return false // Public repo or app installed - not blocked
    }

    // Private repo without app - blocked unless user has push access
    if (!github?.connected) {
      return true // No GitHub = can't push, can't fork, no bot = blocked
    }

    return !github.canPush // Has GitHub but no push access = blocked
  }

  /**
   * Get the reason why submission is blocked (if any)
   */
  private get blockedReason(): string | null {
    if (!this.isBlockedPrivateRepo) return null

    const hasGitHub = this.authStatus?.github?.connected
    if (hasGitHub) {
      return "This is a private repository and you don't have push access. Ask a repository admin to install the Stencila GitHub App."
    }
    return 'This is a private repository without the Stencila GitHub App installed. Reviews cannot be submitted without the app.'
  }

  /**
   * Whether to show the GitHub connect button
   */
  private get showGitHubConnect(): boolean {
    // Don't show if already connected
    if (this.authStatus?.github?.connected) return false

    // Show if: anonymous not allowed OR has Stencila account (who might want attribution)
    const allowAnon = this.authStatus?.reviewConfig?.allowAnonymous ?? false
    const wantsGitHub = !allowAnon || !!this.authStatus?.user
    if (!wantsGitHub) return false

    // Stencila users can always connect via profile (regardless of repo visibility)
    if (this.authStatus?.user) return true

    // Non-Stencila users: OAuth is NOT available for private repos
    // They must sign in with Stencila to access private repos
    if (this.authStatus?.repo?.isPrivate) return false

    // Only show if OAuth is available (Stencila-hosted site, public repo)
    return isStencilaHosted()
  }

  /**
   * Whether the user needs to sign in with Stencila (not just GitHub OAuth)
   *
   * Per spec, private repos can still allow anonymous/bot PRs if:
   * - anon attribute is true (anonymous submissions allowed), AND
   * - GitHub App is installed on the repo
   */
  private get requiresStencilaSignIn(): boolean {
    // Already has Stencila account
    if (this.authStatus?.user) return false

    // Already has GitHub connected
    if (this.authStatus?.github?.connected) return false

    const repo = this.authStatus?.repo

    // Private repo: check if bot PR is possible
    if (repo?.isPrivate) {
      // If app installed AND anonymous allowed → bot can create PR, no sign-in required
      const allowAnon = this.authStatus?.reviewConfig?.allowAnonymous ?? false
      if (repo.appInstalled && allowAnon) {
        return false
      }
      // Otherwise, private repo requires Stencila sign-in
      return true
    }

    // Public repo: check site access requirements
    const allowPublic = this.authStatus?.reviewConfig?.allowPublic ?? false
    if (allowPublic) return false

    // Need site access but don't have it
    return !this.authStatus?.hasSiteAccess
  }

  /**
   * Get description of who will author the PR
   *
   * Decision tree:
   * 1. GitHub connected + canPush → Direct PR as user
   * 2. GitHub connected + !canPush + private repo → Bot PR (can't fork private repos)
   * 3. GitHub connected + !canPush + public repo → Fork PR as user
   * 4. Stencila user without GitHub → Bot PR attributed to user
   * 5. Anonymous → Bot PR
   */
  private get prAuthorDescription(): string {
    const github = this.authStatus?.github
    const user = this.authStatus?.user
    const repo = this.authStatus?.repo

    if (github?.connected) {
      // Has push access - direct PR regardless of repo visibility
      if (github.canPush) {
        return `Pull request will be created as @${github.username}`
      }
      // No push access on private repo - must use bot (can't fork private repos)
      if (repo?.isPrivate) {
        return `Pull request will be created by Stencila bot, attributed to @${github.username}`
      }
      // No push access on public repo - fork to user's account
      return `Pull request will be created as @${github.username} from a fork`
    }

    // Stencila user without GitHub - bot PR with attribution
    if (user) {
      return `Pull request will be created by Stencila bot, attributed to ${user.name}`
    }

    // Anonymous - bot PR without attribution
    return 'Pull request will be created by Stencila bot'
  }

  // =========================================================================
  // GitHub Connect
  // =========================================================================

  /**
   * Open GitHub OAuth connect flow in new tab
   */
  private connectGitHub() {
    window.open(this.gitHubConnectUrl, '_blank')
  }

  // =========================================================================
  // Source Info Methods
  // =========================================================================

  /**
   * Get source info (repository, commit) from the closest root element
   */
  private getSourceInfoFromRoot(): SourceInfo | null {
    const root = document.querySelector('[root]')
    if (!root) {
      console.warn('[SiteReview] No [root] element found')
      return null
    }

    const repository = root.getAttribute('repository')
    const commit = root.getAttribute('commit')

    if (!repository || !commit) {
      console.warn('[SiteReview] Root element missing source attributes:', {
        repository,
        commit,
      })
      return null
    }

    return { repository, commit }
  }

  /**
   * Get the path attribute from the closest root element
   */
  private getPathFromRoot(): string | null {
    const root = document.querySelector('[root]')
    return root?.getAttribute('path') ?? null
  }

  /**
   * Check if source info (repository, commit) matches the current page's source
   */
  private checkSourceConsistency(currentSource: SourceInfo): boolean {
    if (!this.sourceInfo) {
      return true
    }

    return (
      this.sourceInfo.repository === currentSource.repository &&
      this.sourceInfo.commit === currentSource.commit
    )
  }

  // =========================================================================
  // Highlight Management Methods
  // =========================================================================

  /**
   * Handle Glide navigation end - reapply highlights for new page
   */
  private handleGlideEnd = (_event: Event) => {
    const customEvent = _event as CustomEvent<GlideEventDetail>
    console.log('[SiteReview] Navigation to:', customEvent.detail.url)

    // Re-apply highlights for the new page and auto-expand its group
    requestAnimationFrame(() => {
      this.applyHighlights()
      this.expandCurrentPageGroup()

      // Handle pending activation from cross-page navigation
      if (this.pendingActivation !== null) {
        const item = this.pendingItems[this.pendingActivation]
        if (item && getPathname(item.url) === window.location.pathname) {
          this.activateAndScrollToItem(this.pendingActivation, item)
        }
        this.pendingActivation = null
      }

      // Trigger re-render to update current page state
      this.requestUpdate()
    })
  }

  /**
   * Apply highlights for all items on the current page using CSS Custom Highlight API
   */
  private applyHighlights() {
    this.clearHighlights()

    const currentPath = window.location.pathname
    const commentRanges: Range[] = []
    const suggestionRanges: Range[] = []

    this.pendingItems.forEach((item, index) => {
      // Only highlight items on current page
      if (getPathname(item.url) !== currentPath) return

      // Skip page-level comments (no nodeId)
      if (!item.start.nodeId) return

      // Check if this is a media item
      if (this.isMediaItem(item)) {
        const element = document.getElementById(item.start.nodeId)
        if (
          element?.matches(
            'stencila-image-object, stencila-audio-object, stencila-video-object',
          )
        ) {
          element.classList.add('review-has-comment')
        }
        return
      }

      // Text item - create range for highlight
      if (!supportsHighlightAPI()) return

      const range = createRangeForItem(item)
      if (!range) return

      // Track range for click detection
      this.highlightRanges.push({ range, itemIndex: index, type: item.type })

      // Add to appropriate highlight group
      if (item.type === 'comment') {
        commentRanges.push(range)
      } else {
        suggestionRanges.push(range)
      }
    })

    // Create and register CSS highlights
    if (supportsHighlightAPI()) {
      if (commentRanges.length > 0) {
        this.commentHighlight = new Highlight(...commentRanges)
        CSS.highlights.set('review-comment', this.commentHighlight)
      }

      if (suggestionRanges.length > 0) {
        this.suggestionHighlight = new Highlight(...suggestionRanges)
        CSS.highlights.set('review-suggestion', this.suggestionHighlight)
      }
    }

    // Add click listener for highlight detection
    document.addEventListener('click', this.handleDocumentClick)
  }

  /**
   * Clear all document highlights
   */
  private clearHighlights() {
    if (supportsHighlightAPI()) {
      CSS.highlights.delete('review-comment')
      CSS.highlights.delete('review-suggestion')
      CSS.highlights.delete('review-comment-active')
      CSS.highlights.delete('review-suggestion-active')
    }

    // Clear media comment indicators
    document
      .querySelectorAll('.review-has-comment')
      .forEach((el) => el.classList.remove('review-has-comment'))

    this.commentHighlight = null
    this.suggestionHighlight = null
    this.activeHighlight = null
    this.highlightRanges = []

    document.removeEventListener('click', this.handleDocumentClick)
  }

  /**
   * Show a highlight for the current selection while input is open
   */
  private showInputHighlight() {
    this.clearInputHighlight()

    if (!supportsHighlightAPI() || !this.currentSelection) return

    // Create range from current selection
    const { start, end } = this.currentSelection
    if (!start.nodeId || !end.nodeId) return

    const startEl = document.getElementById(start.nodeId)
    const endEl = document.getElementById(end.nodeId)
    if (!startEl || !endEl) return

    const startPos = findTextPosition(startEl, start.offset)
    const endPos = findTextPosition(endEl, end.offset)
    if (!startPos || !endPos) return

    try {
      const range = document.createRange()
      range.setStart(startPos.node, startPos.offset)
      range.setEnd(endPos.node, endPos.offset)

      this.inputHighlight = new Highlight(range)
      const highlightName =
        this.inputType === 'comment'
          ? 'review-comment-active'
          : 'review-suggestion-active'
      CSS.highlights.set(highlightName, this.inputHighlight)
    } catch (e) {
      console.warn('[SiteReview] Failed to create input highlight:', e)
    }
  }

  /**
   * Clear the input highlight
   */
  private clearInputHighlight() {
    if (supportsHighlightAPI() && this.inputHighlight) {
      CSS.highlights.delete('review-comment-active')
      CSS.highlights.delete('review-suggestion-active')
    }
    this.inputHighlight = null
  }

  /**
   * Handle click on document to detect clicks on highlights
   */
  private handleDocumentClick = (e: MouseEvent) => {
    // Don't process clicks inside the review component
    if ((e.target as Element)?.closest('stencila-site-review')) {
      return
    }

    // Check if click is within any highlight range
    const selection = window.getSelection()
    if (!selection) return

    // Create a collapsed range at click position (cross-browser)
    const clickRange = caretRangeFromPoint(e.clientX, e.clientY)
    if (!clickRange) return

    // Check each highlight range
    for (const { range, itemIndex } of this.highlightRanges) {
      if (rangeContainsPoint(range, clickRange)) {
        e.preventDefault()
        e.stopPropagation()
        this.setActiveHighlight(itemIndex)

        // Ensure page group is expanded
        const item = this.pendingItems[itemIndex]
        if (item) {
          const itemPath = getPathname(item.url)
          if (!this.expandedPageGroups.has(itemPath)) {
            this.expandedPageGroups.add(itemPath)
            this.expandedPageGroups = new Set(this.expandedPageGroups)
          }
        }

        // Open the panel if not already open
        if (!this.isOpen) {
          this.isOpen = true
        }

        // Scroll the item into view in the panel after render
        this.updateComplete.then(() => {
          const itemElement = this.shadowRoot?.querySelector(
            `.review-item[data-index="${itemIndex}"]`,
          )
          itemElement?.scrollIntoView({ behavior: 'smooth', block: 'nearest' })
        })
        return
      }
    }
  }

  /**
   * Check if a review item is a media comment (comment on image/audio/video)
   */
  private isMediaItem(item: ReviewItem): boolean {
    return (
      !item.selected &&
      item.start.nodeId === item.end.nodeId &&
      item.start.offset === 0 &&
      item.end.offset === 0 &&
      item.start.nodeId !== ''
    )
  }

  /**
   * Clear any active media highlight classes
   */
  private clearActiveMediaHighlight() {
    document
      .querySelectorAll('.review-active')
      .forEach((el) => el.classList.remove('review-active'))
  }

  /**
   * Set the active highlight for a selected item
   */
  private setActiveHighlight(itemIndex: number | null) {
    // Clear previous active highlight (both text and media)
    if (supportsHighlightAPI()) {
      CSS.highlights.delete('review-comment-active')
      CSS.highlights.delete('review-suggestion-active')
    }
    this.activeHighlight = null
    this.clearActiveMediaHighlight()
    this.activeItemIndex = itemIndex

    if (itemIndex === null) return

    const item = this.pendingItems[itemIndex]
    if (!item?.start.nodeId) return

    // Check if this is a media item
    if (this.isMediaItem(item)) {
      const element = document.getElementById(item.start.nodeId)
      if (
        element?.matches(
          'stencila-image-object, stencila-audio-object, stencila-video-object',
        )
      ) {
        element.classList.add('review-active')
      }
      return
    }

    // Create range for active text item
    const range = createRangeForItem(item)
    if (!range) return

    // Create active highlight
    if (supportsHighlightAPI()) {
      this.activeHighlight = new Highlight(range)
      const highlightName =
        item.type === 'comment'
          ? 'review-comment-active'
          : 'review-suggestion-active'
      CSS.highlights.set(highlightName, this.activeHighlight)
    }
  }

  /**
   * Get the current header height from CSS variable or 0 if no header
   */
  private getHeaderOffset(): number {
    // Check if header exists
    const header = document.querySelector('stencila-header')
    if (!header) {
      return 0
    }

    // Get the computed header height from CSS variable
    const headerHeight = getComputedStyle(document.documentElement)
      .getPropertyValue('--layout-header-height')
      .trim()

    // Parse the value (e.g., "64px" -> 64)
    if (headerHeight) {
      const parsed = parseFloat(headerHeight)
      if (!isNaN(parsed)) {
        return parsed
      }
    }

    // Fallback: use the header's actual height
    return header.getBoundingClientRect().height
  }

  /**
   * Activate an item and scroll to its highlight in the document (if needed)
   */
  private activateAndScrollToItem(index: number, item: ReviewItem) {
    this.setActiveHighlight(index)

    if (item.start.nodeId) {
      const element = document.getElementById(item.start.nodeId)
      if (element) {
        const headerOffset = this.getHeaderOffset()
        const rect = element.getBoundingClientRect()
        const viewportHeight = window.innerHeight

        // Check if element is already visible in viewport (accounting for header)
        const isVisible =
          rect.top >= headerOffset && rect.bottom <= viewportHeight

        // Only scroll if element is not visible
        if (!isVisible) {
          const offsetPosition = rect.top + window.scrollY - headerOffset
          window.scrollTo({ top: offsetPosition, behavior: 'smooth' })
        }
      }
    }
  }

  /**
   * Handle clicking on a review item in the panel
   */
  private handleItemClick(index: number, item: ReviewItem) {
    const itemPath = getPathname(item.url)
    const currentPath = window.location.pathname

    if (itemPath !== currentPath) {
      // Navigate to the other page first, store pending activation
      this.pendingActivation = index
      navigate(itemPath, 'click')
    } else {
      // Same page - just activate and scroll
      this.activateAndScrollToItem(index, item)
    }
  }

  // =========================================================================
  // Page Grouping Methods
  // =========================================================================

  /**
   * Group pending items by page URL
   */
  private get itemsByPage(): Map<
    string,
    { items: ReviewItem[]; indices: number[] }
  > {
    const groups = new Map<
      string,
      { items: ReviewItem[]; indices: number[] }
    >()

    this.pendingItems.forEach((item, index) => {
      const path = getPathname(item.url)
      if (!groups.has(path)) {
        groups.set(path, { items: [], indices: [] })
      }
      const group = groups.get(path)
      if (group) {
        group.items.push(item)
        group.indices.push(index)
      }
    })

    return groups
  }

  /**
   * Check if a page path is the current page
   */
  private isCurrentPage(path: string): boolean {
    return path === window.location.pathname
  }

  /**
   * Toggle a page group's expanded state
   */
  private togglePageGroup(path: string) {
    if (this.expandedPageGroups.has(path)) {
      this.expandedPageGroups.delete(path)
    } else {
      this.expandedPageGroups.add(path)
    }
    // Trigger re-render with new Set
    this.expandedPageGroups = new Set(this.expandedPageGroups)
  }

  /**
   * Auto-expand the page group for the current page
   */
  private expandCurrentPageGroup() {
    const currentPath = window.location.pathname
    // Only expand if there are items for this page
    const hasItemsForCurrentPage = this.pendingItems.some(
      (item) => getPathname(item.url) === currentPath,
    )
    if (hasItemsForCurrentPage && !this.expandedPageGroups.has(currentPath)) {
      this.expandedPageGroups.add(currentPath)
      this.expandedPageGroups = new Set(this.expandedPageGroups)
    }
  }

  /**
   * Handle selection changes
   */
  private handleSelectionChange = () => {
    // We process on mouseup instead to get the final selection
  }

  /**
   * Handle mouseup to capture final selection or media click
   */
  private handleMouseUp = (e: MouseEvent) => {
    // Don't process selection if clicking inside the site-review component
    // (e.g., on the modal or floating button) - use composedPath for Shadow DOM
    const path = e.composedPath()
    if (path.includes(this)) {
      return
    }

    // Don't process selection if input modal is open
    if (this.showInput) {
      return
    }

    // Check if clicking on a media element (image, audio, video)
    const target = e.target as Element
    const mediaElement = target.closest(
      'stencila-image-object, stencila-audio-object, stencila-video-object',
    )

    if (mediaElement && mediaElement.id) {
      // Clear any text selection
      this.currentSelection = null
      window.getSelection()?.removeAllRanges()

      // Check if this media element already has a comment
      if (mediaElement.classList.contains('review-has-comment')) {
        // Find the item for this media element and activate it
        const itemIndex = this.pendingItems.findIndex(
          (item) =>
            this.isMediaItem(item) && item.start.nodeId === mediaElement.id,
        )

        if (itemIndex >= 0) {
          this.setActiveHighlight(itemIndex)

          // Ensure page group is expanded
          const item = this.pendingItems[itemIndex]
          const itemPath = getPathname(item.url)
          if (!this.expandedPageGroups.has(itemPath)) {
            this.expandedPageGroups.add(itemPath)
            this.expandedPageGroups = new Set(this.expandedPageGroups)
          }

          // Open the panel if not already open
          if (!this.isOpen) {
            this.isOpen = true
          }

          // Scroll the item into view in the panel after render
          this.updateComplete.then(() => {
            const itemElement = this.querySelector(
              `.review-item[data-index="${itemIndex}"]`,
            )
            itemElement?.scrollIntoView({ behavior: 'smooth', block: 'nearest' })
          })

          console.log('[SiteReview] Activated media comment:', itemIndex)
          return
        }
      }

      // No existing comment - create media selection for new comment
      this.currentMediaSelection = {
        nodeId: mediaElement.id,
        element: mediaElement,
        rect: mediaElement.getBoundingClientRect(),
        mediaType: mediaElement.tagName.toLowerCase().replace('stencila-', ''),
      }

      // Add visual indicator
      this.clearMediaSelectionClass()
      mediaElement.classList.add('review-selected')

      console.log('[SiteReview] Media selection:', {
        nodeId: this.currentMediaSelection.nodeId,
        mediaType: this.currentMediaSelection.mediaType,
      })
      return
    }

    // Not clicking on media - clear any media selection
    this.clearMediaSelection()

    // Small delay to ensure selection is finalized
    setTimeout(() => this.processSelection(), 10)
  }

  /**
   * Clear media selection visual class from all elements
   */
  private clearMediaSelectionClass() {
    document
      .querySelectorAll('.review-selected')
      .forEach((el) => el.classList.remove('review-selected'))
  }

  /**
   * Clear the current media selection
   */
  private clearMediaSelection() {
    if (this.currentMediaSelection) {
      this.clearMediaSelectionClass()
      this.currentMediaSelection = null
    }
  }

  /**
   * Handle scroll to update floating button/popover position
   */
  private handleScroll = () => {
    // Update media selection rect on scroll
    if (this.currentMediaSelection) {
      const element = this.currentMediaSelection.element
      if (element && document.contains(element)) {
        this.currentMediaSelection = {
          ...this.currentMediaSelection,
          rect: element.getBoundingClientRect(),
        }
      } else {
        this.clearMediaSelection()
      }
      return
    }

    if (!this.currentSelection) return

    // Recreate range to get updated viewport-relative rect
    const { start, end } = this.currentSelection
    if (!start.nodeId || !end.nodeId) return

    const startEl = document.getElementById(start.nodeId)
    const endEl = document.getElementById(end.nodeId)
    if (!startEl || !endEl) return

    const startPos = findTextPosition(startEl, start.offset)
    const endPos = findTextPosition(endEl, end.offset)
    if (!startPos || !endPos) return

    try {
      const range = document.createRange()
      range.setStart(startPos.node, startPos.offset)
      range.setEnd(endPos.node, endPos.offset)

      this.currentSelection = {
        ...this.currentSelection,
        rect: range.getBoundingClientRect(),
      }
    } catch {
      this.currentSelection = null
    }
  }

  /**
   * Process the current selection
   */
  private processSelection() {
    const selection = window.getSelection()
    if (!selection || selection.isCollapsed || !selection.rangeCount) {
      this.currentSelection = null
      return
    }

    const range = selection.getRangeAt(0)
    const selectedText = range.toString().trim()

    if (!selectedText) {
      this.currentSelection = null
      return
    }

    // Check selection length against min/max limits
    if (selectedText.length < this.minSelection) {
      this.currentSelection = null
      return
    }

    if (selectedText.length > this.maxSelection) {
      // Selection too large - could show warning in future
      console.log(
        '[SiteReview] Selection exceeds max length:',
        selectedText.length,
      )
      this.currentSelection = null
      return
    }

    // Find start anchor
    const startNode = findNodeWithId(range.startContainer)
    if (!startNode) {
      this.currentSelection = null
      return
    }
    const startOffset = getCharOffset(
      startNode,
      range.startContainer,
      range.startOffset,
    )

    // Find end anchor (may be different node for multi-block)
    const endNode = findNodeWithId(range.endContainer)
    if (!endNode) {
      this.currentSelection = null
      return
    }
    const endOffset = getCharOffset(
      endNode,
      range.endContainer,
      range.endOffset,
    )

    const rect = range.getBoundingClientRect()

    this.currentSelection = {
      start: { nodeId: startNode.id, offset: startOffset },
      end: { nodeId: endNode.id, offset: endOffset },
      selectedText,
      rect,
    }

    // Log for debugging (walking skeleton)
    console.log('[SiteReview] Selection:', {
      start: this.currentSelection.start,
      end: this.currentSelection.end,
      selectedText:
        selectedText.length > 50
          ? selectedText.slice(0, 50) + '...'
          : selectedText,
      isMultiBlock: startNode.id !== endNode.id,
    })
  }

  /**
   * Handle adding a page-level comment (no text selection required)
   */
  private handlePageComment() {
    this.inputType = 'comment'
    this.currentSelection = null // Clear any selection
    this.showInput = true
  }

  /**
   * Handle clicking the comment button
   */
  private handleComment() {
    this.inputType = 'comment'
    this.showInput = true
    this.showInputHighlight()
  }

  /**
   * Handle clicking the suggest button
   */
  private handleSuggest() {
    this.inputType = 'suggestion'
    this.showInput = true
    this.showInputHighlight()
  }

  /**
   * Handle clicking the comment button on a media element
   */
  private handleMediaComment() {
    this.inputType = 'comment'
    this.showInput = true
    // Don't show input highlight for media - the element is already visually selected
  }

  /**
   * Handle canceling the input
   */
  private handleCancel() {
    this.showInput = false
    this.clearInputHighlight()
    this.currentSelection = null
    this.clearMediaSelection()
    window.getSelection()?.removeAllRanges()
  }

  /**
   * Handle item-delete event from child component
   */
  private handleItemDelete(e: CustomEvent<ItemDeleteDetail>) {
    const { index } = e.detail

    // Clear active highlight if deleting the active item
    if (this.activeItemIndex === index) {
      this.setActiveHighlight(null)
    } else if (this.activeItemIndex !== null && this.activeItemIndex > index) {
      // Adjust active index if deleting an item before it
      this.activeItemIndex--
    }

    this.pendingItems = this.pendingItems.filter((_, i) => i !== index)
    this.saveToStorage()

    // Reapply highlights after deletion
    this.applyHighlights()
  }

  /**
   * Handle item-edit event from child component
   */
  private handleItemEdit(e: CustomEvent<ItemEditDetail>) {
    const { index, content } = e.detail

    const updatedItems = [...this.pendingItems]
    if (updatedItems[index]) {
      updatedItems[index] = {
        ...updatedItems[index],
        content,
      }
      this.pendingItems = updatedItems
      this.saveToStorage()
    }
  }

  /**
   * Handle item-add event from child component
   */
  private handleItemAdd(e: CustomEvent<ItemAddDetail>) {
    const { type, content, selection } = e.detail

    // Get source info from the current page's root element
    const currentSource = this.getSourceInfoFromRoot()
    if (!currentSource) {
      this.errorMessage =
        'Unable to determine source information for this page. Please contact the site owner.'
      this.showInput = false
      return
    }

    // Check consistency with existing review (skip on localhost for dev convenience)
    if (!isLocalhost() && !this.checkSourceConsistency(currentSource)) {
      this.showCommitMismatch = true
      this.showInput = false
      return
    }

    // On localhost with commit mismatch, silently update to current source
    if (isLocalhost() && !this.checkSourceConsistency(currentSource)) {
      console.log(
        '[SiteReview] Commit mismatch on localhost, updating source info',
      )
      this.sourceInfo = currentSource
    }

    // Set source info if this is the first item
    if (!this.sourceInfo) {
      this.sourceInfo = currentSource
      console.log('[SiteReview] Set source info:', this.sourceInfo)
    }

    // Get the path for this page
    const path = this.getPathFromRoot()
    if (!path) {
      this.errorMessage =
        'Unable to determine source path for this page. Please contact the site owner.'
      this.showInput = false
      return
    }

    // Build the review item - handle selection-based, page-level, and media comments
    const isMediaComment = this.currentMediaSelection !== null
    const isPageComment = !selection && !isMediaComment

    let itemStart: ReviewItemAnchor
    let itemEnd: ReviewItemAnchor
    let itemSelected: string

    if (isMediaComment) {
      // Media comment: use media element's nodeId with offset 0
      itemStart = { nodeId: this.currentMediaSelection!.nodeId, offset: 0 }
      itemEnd = { nodeId: this.currentMediaSelection!.nodeId, offset: 0 }
      itemSelected = ''
    } else if (isPageComment) {
      // Page-level comment: empty nodeId
      itemStart = { nodeId: '', offset: 0 }
      itemEnd = { nodeId: '', offset: 0 }
      itemSelected = ''
    } else {
      // Text selection comment/suggestion
      itemStart = selection.start
      itemEnd = selection.end
      itemSelected = selection.selectedText
    }

    const item: ReviewItem = {
      type,
      path,
      url: window.location.origin + window.location.pathname,
      title: document.title || window.location.pathname,
      start: itemStart,
      end: itemEnd,
      selected: itemSelected,
      content,
    }

    // Clear any previous submission state when adding new items
    this.submitResult = null
    this.submitError = ''

    // Insert item in position order among items for the same URL
    const insertIndex = this.findInsertPosition(item)
    const newItems = [...this.pendingItems]
    newItems.splice(insertIndex, 0, item)
    this.pendingItems = newItems
    this.saveToStorage()

    // Clear input highlight before reapplying all highlights
    this.clearInputHighlight()

    // Reapply highlights to include the new item
    this.applyHighlights()

    // Make the new item the active item
    this.setActiveHighlight(insertIndex)

    // Auto-expand the current page group so the new item is visible
    this.expandCurrentPageGroup()

    // Close input and trigger FAB pulse
    this.showInput = false
    this.currentSelection = null
    this.clearMediaSelection()

    this.isFabPulsing = true
    setTimeout(() => {
      this.isFabPulsing = false
    }, 400)

    // Clear browser selection
    window.getSelection()?.removeAllRanges()

    // Log for debugging
    console.log('[SiteReview] Added item:', item)
    console.log('[SiteReview] Pending items:', this.pendingItems.length)
  }

  /**
   * Handle item-cancel event from child component
   */
  private handleItemCancel() {
    this.showInput = false
    this.clearInputHighlight()
    this.currentSelection = null
    this.clearMediaSelection()
    window.getSelection()?.removeAllRanges()
  }

  /**
   * Handle item-click event from child component
   */
  private handleItemClickEvent(e: CustomEvent<ItemClickDetail>) {
    const { index, item } = e.detail
    this.handleItemClick(index, item)
  }

  /**
   * Handle clearing old review due to commit mismatch
   */
  private handleClearForNewCommit() {
    this.pendingItems = []
    this.sourceInfo = null
    this.activeItemIndex = null
    this.saveToStorage()
    this.showCommitMismatch = false

    // Clear all document highlights
    this.clearHighlights()
  }

  /**
   * Dismiss the commit mismatch modal without clearing
   */
  private dismissCommitMismatch() {
    this.showCommitMismatch = false
  }

  /**
   * Handle errors from review submission
   */
  private handleSubmitError(error: ApiError, statusCode?: number) {
    // Handle HTTP status codes
    if (statusCode === 401) {
      this.submitError =
        'Authentication required. Please sign in and try again.'
      return
    }
    if (statusCode === 403) {
      this.submitError = 'Reviews are disabled for this workspace.'
      return
    }
    if (statusCode === 429) {
      this.submitError = `Too many requests. Please try again in ${error.retryAfter ?? 60} seconds.`
      return
    }

    // Handle error codes from response body
    switch (error.error) {
      case 'rate_limited':
        this.submitError = `Too many requests. Please try again in ${error.retryAfter ?? 60} seconds.`
        break
      case 'private_repo_no_app':
        this.submitError =
          'Cannot submit review: this private repository requires the Stencila GitHub App to be installed.'
        break
      case 'private_repo_oauth_rejected':
        this.submitError =
          'Cannot submit review: private repositories require signing in with Stencila.'
        break
      case 'private_repo_no_auth':
        this.submitError =
          'Cannot submit review: this private repository requires authentication. Please sign in.'
        break
      case 'fork_failed':
        this.submitError =
          'Failed to create fork for pull request. Please try again later.'
        break
      default:
        this.submitError =
          error.message ?? 'Failed to submit review. Please try again.'
    }
  }

  /**
   * Handle submitting the full review
   */
  private async handleSubmitReview() {
    if (this.pendingItems.length === 0 || !this.sourceInfo) {
      return
    }

    // Check if user can submit
    if (!this.canSubmitReview) {
      // If not authenticated and anonymous not allowed, prompt for GitHub
      if (this.showGitHubConnect) {
        this.connectGitHub()
      }
      return
    }

    this.submitting = true
    this.submitError = ''

    // Generate share URL for the review
    const shareResult = await encodeReviewForUrl(
      this.pendingItems,
      this.sourceInfo ?? undefined,
    )
    const shareUrl = new URL(window.location.href)
    shareUrl.searchParams.set(SHARE_PARAM, shareResult.encoded)

    // On localhost preview, show the payload that would be submitted
    if (isLocalhost()) {
      this.showPreviewMock({
        endpoint: REVIEW_SUBMIT_PATH,
        method: 'POST',
        body: {
          commit: this.sourceInfo.commit,
          items: this.pendingItems,
          authorAsSelf: true,
          shareUrl: shareUrl.toString(),
        },
      })

      // Clear pending items
      this.pendingItems = []
      this.sourceInfo = null
      this.saveToStorage()
      this.submitting = false
      return
    }

    try {
      const response = await this.apiFetch(REVIEW_SUBMIT_PATH, {
        method: 'POST',
        body: {
          commit: this.sourceInfo.commit,
          items: this.pendingItems,
          authorAsSelf: true,
          shareUrl: shareUrl.toString(),
        },
      })

      if (!response.ok) {
        const error: ApiError = await response.json()
        this.handleSubmitError(error, response.status)
        return
      }

      const result: ReviewResponse = await response.json()
      console.log('[SiteReview] Review submitted:', result)

      // Show success result
      this.submitResult = result

      // Clear pending items after successful submission
      this.pendingItems = []
      this.sourceInfo = null
      this.saveToStorage()
    } catch (e) {
      console.error('[SiteReview] Submit failed:', e)
      this.submitError =
        'Failed to submit review. Please check your connection and try again.'
    } finally {
      this.submitting = false
    }
  }

  override render() {
    return html`
      ${this.renderReviewFab()} ${this.renderSelection()} ${this.renderInput()}
      ${this.renderReviewPanel()} ${this.renderErrorModal()}
      ${this.renderCommitMismatchModal()} ${this.renderPreviewMockModal()}
    `
  }

  /**
   * Render the FAB (Floating Action Button) to toggle the review panel
   * Uses shared CSS classes with review-specific pulsing animation
   */
  private renderReviewFab() {
    // Don't render FAB if inside a site-actions container or user not allowed
    if (this.hideFab || (!this.authLoading && !this.isActionAllowed)) {
      return nothing
    }

    const itemCount = this.pendingItems.length

    return html`
      <button
        class="site-action-fab ${this.isFabPulsing ? 'pulsing' : ''}"
        style=${this.fabPositionStyles}
        @click=${this.togglePanel}
        aria-label=${itemCount > 0
          ? `Open review panel (${itemCount} pending)`
          : 'Open review panel'}
        aria-expanded=${this.isOpen}
      >
        <span class="i-lucide:message-square-plus fab-icon"></span>
        ${itemCount > 0
          ? html`<span class="site-action-badge"
              >${itemCount > 99 ? '99+' : itemCount}</span
            >`
          : nothing}
      </button>
    `
  }

  /**
   * Render the floating button that appears near text selection or media element
   * Positioned at the end of the selection (where user's cursor likely is)
   */
  private renderSelection() {
    if (this.showInput) {
      return nothing
    }

    // Handle media selection (only show Comment button)
    if (this.currentMediaSelection) {
      const rect = this.currentMediaSelection.rect
      const buttonWidth = 40 // Single button width
      const margin = 8
      const left = Math.max(
        margin,
        Math.min(
          rect.right - buttonWidth,
          window.innerWidth - buttonWidth - margin,
        ),
      )

      return html`
        <div
          class="floating-button"
          style="top: ${rect.bottom + margin}px; left: ${left}px;"
        >
          ${this.allowsComments
            ? html`<button
                @click=${this.handleMediaComment}
                aria-label="Add comment on media"
              >
                <span class="i-lucide:message-circle"></span>
              </button>`
            : null}
        </div>
      `
    }

    // Handle text selection
    if (!this.currentSelection) {
      return nothing
    }

    // Position with right edge aligned to selection end, clamped within viewport
    const rect = this.currentSelection.rect
    const buttonWidth = 76 // Approximate width of button group
    const margin = 8
    const left = Math.max(
      margin,
      Math.min(
        rect.right - buttonWidth,
        window.innerWidth - buttonWidth - margin,
      ),
    )

    return html`
      <div
        class="floating-button"
        style="top: ${rect.bottom + margin}px; left: ${left}px;"
      >
        ${this.allowsComments
          ? html`<button @click=${this.handleComment} aria-label="Add comment">
              <span class="i-lucide:message-circle"></span>
            </button>`
          : null}
        ${this.allowsSuggestions
          ? html`<button
              @click=${this.handleSuggest}
              aria-label="Suggest change"
            >
              <span class="i-lucide:pencil"></span>
            </button>`
          : null}
      </div>
    `
  }

  /**
   * Get the current selection as SelectionInfo for the child component
   */
  private get selectionInfo(): SelectionInfo | null {
    if (!this.currentSelection) return null
    return {
      start: this.currentSelection.start,
      end: this.currentSelection.end,
      selectedText: this.currentSelection.selectedText,
    }
  }

  /**
   * Calculate smart popover position based on selection rect
   * - Positions below selection by default
   * - Flips above if near bottom of viewport
   * - Shifts horizontally to stay within viewport
   * - Returns arrow position for visual connection to selection
   */
  private calculatePopoverPosition(): {
    top: number
    left: number
    maxWidth: number
    arrow: 'top' | 'bottom'
    arrowLeft: number
  } | null {
    if (!this.currentSelection) return null

    const rect = this.currentSelection.rect
    const viewportHeight = window.innerHeight
    const viewportWidth = window.innerWidth

    const popoverHeight = 160 // Approximate height of popover
    const popoverMinWidth = 280
    const popoverMaxWidth = 400
    const margin = 8
    const arrowSize = 8

    // Calculate vertical position (using viewport coordinates for fixed positioning)
    let top: number
    let arrow: 'top' | 'bottom'
    const spaceBelow = viewportHeight - rect.bottom
    const spaceAbove = rect.top

    if (spaceBelow >= popoverHeight + margin + arrowSize) {
      // Position below selection - arrow points up from top edge
      top = rect.bottom + margin + arrowSize
      arrow = 'top'
    } else if (spaceAbove >= popoverHeight + margin + arrowSize) {
      // Position above selection - arrow points down from bottom edge
      top = rect.top - popoverHeight - margin - arrowSize
      arrow = 'bottom'
    } else {
      // Not enough space either way, position below anyway
      top = rect.bottom + margin + arrowSize
      arrow = 'top'
    }

    // Calculate horizontal position - align with selection start
    let left = rect.left
    const maxWidth = Math.min(popoverMaxWidth, viewportWidth - margin * 2)

    // Ensure popover doesn't go off right edge
    if (left + popoverMinWidth > viewportWidth - margin) {
      left = viewportWidth - popoverMinWidth - margin
    }

    // Ensure popover doesn't go off left edge
    if (left < margin) {
      left = margin
    }

    // Calculate arrow horizontal position (center of selection, relative to popover left)
    // Minimum offset accounts for border-radius (~12px) + arrow half-width (8px) = 20px, use 24px for safety
    const selectionCenter = rect.left + rect.width / 2
    const arrowOffset = 24
    const arrowLeft = Math.max(
      arrowOffset,
      Math.min(selectionCenter - left, popoverMinWidth - arrowOffset),
    )

    return { top, left, maxWidth, arrow, arrowLeft }
  }

  /**
   * Render the input modal for adding comments/suggestions
   */
  private renderInput() {
    if (!this.showInput) {
      return nothing
    }

    // Calculate popover position if we have a selection
    const popoverPosition = this.calculatePopoverPosition()

    // Use popover mode when we have selection (inline near text)
    // Use modal mode for page-level comments (centered with backdrop)
    if (popoverPosition) {
      return html`
        <div
          class="site-action-backdrop-transparent"
          @click=${this.handleCancel}
        ></div>
        <stencila-site-review-item
          mode="input"
          type=${this.inputType}
          .selection=${this.selectionInfo}
          .popoverPosition=${popoverPosition}
          page-title=${document.title || window.location.pathname}
          @item-add=${this.handleItemAdd}
          @item-cancel=${this.handleItemCancel}
        ></stencila-site-review-item>
      `
    }

    // Modal mode for page-level comments (no selection)
    return html`
      <div class="site-action-backdrop" @click=${this.handleCancel}></div>
      <stencila-site-review-item
        mode="input"
        type=${this.inputType}
        .selection=${this.selectionInfo}
        page-title=${document.title || window.location.pathname}
        @item-add=${this.handleItemAdd}
        @item-cancel=${this.handleItemCancel}
      ></stencila-site-review-item>
    `
  }

  /**
   * Render the list of pending review items, grouped by page
   * Routes are kept in insertion order (order first item was added for that route)
   * Items within a route are sorted by position in the document
   */
  private renderPanelItems() {
    const itemsByPage = this.itemsByPage

    // Keep routes in insertion order (Map preserves insertion order)
    const paths = Array.from(itemsByPage.keys())

    return html`
      <div class="items-list">
        ${paths.map((path) => {
          const group = itemsByPage.get(path)
          if (!group) return nothing
          return this.renderPageGroup(path, group)
        })}
      </div>
    `
  }

  /**
   * Find the correct insertion index for a new item in pendingItems
   * Items are ordered by URL (insertion order) then by document position within each URL
   */
  private findInsertPosition(newItem: ReviewItem): number {
    const newUrl = newItem.url

    // Find the range of existing items for this URL
    let firstIndexForUrl = -1
    let lastIndexForUrl = -1
    for (let i = 0; i < this.pendingItems.length; i++) {
      if (this.pendingItems[i].url === newUrl) {
        if (firstIndexForUrl === -1) firstIndexForUrl = i
        lastIndexForUrl = i
      }
    }

    // If no items exist for this URL, append at the end
    if (firstIndexForUrl === -1) {
      return this.pendingItems.length
    }

    // Find the correct position within items for this URL
    for (let i = firstIndexForUrl; i <= lastIndexForUrl; i++) {
      const existing = this.pendingItems[i]
      if (this.compareItemPositions(newItem, existing) < 0) {
        // New item comes before existing item
        return i
      }
    }

    // New item comes after all existing items for this URL
    return lastIndexForUrl + 1
  }

  /**
   * Compare two items by their position in the document
   * Returns negative if a comes before b, positive if after, 0 if same
   */
  private compareItemPositions(a: ReviewItem, b: ReviewItem): number {
    // Page-level comments (no nodeId) go at the end
    if (!a.start.nodeId && !b.start.nodeId) return 0
    if (!a.start.nodeId) return 1
    if (!b.start.nodeId) return -1

    // Same node - compare offsets
    if (a.start.nodeId === b.start.nodeId) {
      return a.start.offset - b.start.offset
    }

    // Different nodes - find their DOM order
    const elA = document.getElementById(a.start.nodeId)
    const elB = document.getElementById(b.start.nodeId)

    if (!elA || !elB) {
      // Can't find elements (maybe on different page), keep original order
      return 0
    }

    // Use compareDocumentPosition to determine order
    const position = elA.compareDocumentPosition(elB)
    if (position & Node.DOCUMENT_POSITION_FOLLOWING) {
      return -1 // a comes before b
    }
    if (position & Node.DOCUMENT_POSITION_PRECEDING) {
      return 1 // a comes after b
    }
    return 0
  }

  /**
   * Render a collapsible page group
   * Current page is auto-expanded on navigation but can be collapsed
   */
  private renderPageGroup(
    path: string,
    group: { items: ReviewItem[]; indices: number[] },
  ) {
    const isCurrent = this.isCurrentPage(path)
    // All page groups can be toggled; current page is auto-expanded on navigation
    const isExpanded = this.expandedPageGroups.has(path)

    // Items are already in position order (sorted at insertion time)
    const itemsWithIndices = group.items.map((item, i) => ({
      item,
      index: group.indices[i],
    }))

    // Use title from first item in group, fallback to path
    const pageTitle = group.items[0]?.title || path

    return html`
      <div class="page-group" data-current=${isCurrent}>
        <button
          class="page-group-header"
          @click=${(e: Event) => {
            e.stopPropagation()
            this.togglePageGroup(path)
          }}
          aria-expanded=${isExpanded}
        >
          <span class="page-path">${pageTitle}</span>
          <span
            class="chevron i-lucide:chevron-${isExpanded ? 'up' : 'down'}"
          ></span>
        </button>
        ${isExpanded
          ? html`
              <div class="page-group-items">
                ${itemsWithIndices.map(({ item, index }) =>
                  this.renderReviewItem(item, index),
                )}
              </div>
            `
          : null}
      </div>
    `
  }

  /**
   * Render an individual review item (within a page group)
   */
  private renderReviewItem(item: ReviewItem, index: number) {
    const isActive = this.activeItemIndex === index

    return html`
      <stencila-site-review-item
        .item=${item}
        .index=${index}
        ?active=${isActive}
        @item-click=${this.handleItemClickEvent}
        @item-edit=${this.handleItemEdit}
        @item-delete=${this.handleItemDelete}
      ></stencila-site-review-item>
    `
  }

  /**
   * Render the panel footer with submit button and actions
   */
  private renderPanelFooter() {
    const state = this.calculateFooterState()
    const canSubmit = state.type === 'canSubmit'

    return html`
      <div class="site-action-panel-footer">
        <button
          class="add-comment-fab"
          @click=${(e: Event) => {
            e.stopPropagation()
            this.handlePageComment()
          }}
          aria-label="Add page comment"
        >
          <span class="i-lucide:plus"></span>
        </button>

        <div class="site-action-footer-buttons">
          <button
            class="site-action-btn site-action-btn-primary"
            @click=${this.handleSubmitReview}
            ?disabled=${!canSubmit}
          >
            Submit
          </button>
          <div class="share-btn-container">
            <button
              class="site-action-btn site-action-btn-secondary icon-only"
              @click=${(e: Event) => {
                e.stopPropagation()
                this.handleShare()
              }}
              aria-label="Share review"
              ?disabled=${this.pendingItems.length === 0}
            >
              <span class="i-lucide:share-2"></span>
            </button>
            ${this.shareTooltip
              ? html`<div class="share-tooltip">${this.shareTooltip}</div>`
              : nothing}
          </div>
        </div>

        ${this.renderFooterStatus()}
      </div>
    `
  }

  /**
   * Render the review panel using base class structure with shared CSS classes
   */
  private renderReviewPanel() {
    if (!this.isOpen) {
      return nothing
    }

    return html`
      <div class="site-action-panel" style=${this.panelPositionStyles}>
        ${this.renderPanelHeader()}
        <div class="site-action-panel-content">${this.renderPanelContent()}</div>
      </div>
    `
  }

  /**
   * Render commit mismatch warning modal
   */
  private renderCommitMismatchModal() {
    if (!this.showCommitMismatch) {
      return nothing
    }

    return html`
      <div
        class="site-action-backdrop"
        @click=${this.dismissCommitMismatch}
      ></div>
      <div class="site-action-modal warning">
        <h4>Site Updated</h4>
        <p>
          The site has been updated since you started your review. Your pending
          review items (${this.pendingItems.length}) were created for an older
          version and may no longer apply correctly.
        </p>
        <div class="buttons">
          <button
            class="site-action-btn secondary"
            @click=${this.dismissCommitMismatch}
          >
            Keep Old Review
          </button>
          <button
            class="site-action-btn warning"
            @click=${this.handleClearForNewCommit}
          >
            Clear & Start Fresh
          </button>
        </div>
      </div>
    `
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'stencila-site-review': StencilaSiteReview;
  }
}
