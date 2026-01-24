import { LitElement, html, nothing } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { navigate } from '../../glide'

import { SearchEngine } from './engine'
import type { RecentSearch, SearchEntry, SearchResult } from './types'

/**
 * Site search component
 *
 * Provides a search interface for statically hosted Stencila sites.
 * Features:
 * - Keyboard shortcut (Cmd/Ctrl+K) to open
 * - Debounced search input
 * - Result type badges
 * - Navigation to route#nodeId anchors
 * - Escape to close
 */
@customElement('stencila-site-search')
export class StencilaSiteSearch extends LitElement {
  /**
   * Placeholder text for the search input
   */
  @property({ type: String })
  placeholder = 'Search...'

  /**
   * Debounce delay in milliseconds
   */
  @property({ type: Number, attribute: 'debounce-delay' })
  debounceDelay = 150

  /**
   * Maximum number of results to show
   */
  @property({ type: Number, attribute: 'max-results' })
  maxResults = 10

  /**
   * Base path for the search index
   */
  @property({ type: String, attribute: 'index-path' })
  indexPath = '/_search'

  /**
   * Whether the search modal is open
   */
  @state()
  private isOpen = false

  /**
   * Current search query
   */
  @state()
  private query = ''

  /**
   * Search results
   */
  @state()
  private results: SearchResult[] = []

  /**
   * Whether a search is in progress
   */
  @state()
  private isSearching = false

  /**
   * Currently selected result index for keyboard navigation
   */
  @state()
  private selectedIndex = -1

  /**
   * Error message if search fails
   */
  @state()
  private error: string | null = null

  /**
   * Recent search selections
   */
  @state()
  private recentSearches: RecentSearch[] = []

  /**
   * Search engine instance
   */
  private engine: SearchEngine | null = null

  /**
   * localStorage key for recent searches
   */
  private readonly STORAGE_KEY = 'stencila-site-search-recent'

  /**
   * Maximum number of recent searches to store
   */
  private readonly MAX_RECENT = 5

  /**
   * Debounce timeout
   */
  private debounceTimeout: number | null = null

  /**
   * Search version counter to prevent stale results from overwriting newer ones
   */
  private searchVersion = 0

  /**
   * Use Light DOM so theme CSS can style the component
   */
  protected override createRenderRoot() {
    return this
  }

  override async connectedCallback() {
    super.connectedCallback()

    // Load recent searches from localStorage
    this.loadRecentSearches()

    // Initialize search engine
    this.engine = new SearchEngine(this.indexPath)
    try {
      await this.engine.initialize()
    } catch (e) {
      console.warn('Search index not available:', e)
    }

    // Listen for keyboard shortcuts
    document.addEventListener('keydown', this.handleGlobalKeydown)
  }

  override disconnectedCallback() {
    super.disconnectedCallback()
    document.removeEventListener('keydown', this.handleGlobalKeydown)
    if (this.debounceTimeout) {
      clearTimeout(this.debounceTimeout)
    }
  }

  /**
   * Handle global keyboard events
   */
  private handleGlobalKeydown = (event: KeyboardEvent) => {
    // Cmd/Ctrl+K to open search
    if ((event.metaKey || event.ctrlKey) && event.key === 'k') {
      // Don't intercept in editable elements (inputs, textareas, contenteditable)
      const target = event.target as HTMLElement
      const tagName = target.tagName
      if (
        tagName === 'INPUT' ||
        tagName === 'TEXTAREA' ||
        target.isContentEditable
      ) {
        return
      }

      event.preventDefault()
      this.open()
    }

    // Escape to close
    if (event.key === 'Escape' && this.isOpen) {
      event.preventDefault()
      this.close()
    }
  }

  /**
   * Handle keyboard events in the search input
   */
  private handleInputKeydown = (event: KeyboardEvent) => {
    // Determine which list we're navigating (results or recent searches)
    const isShowingRecent = !this.query.trim() && this.recentSearches.length > 0
    const items = isShowingRecent ? this.recentSearches : this.results
    const maxIndex = items.length - 1

    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault()
        this.selectedIndex = Math.min(this.selectedIndex + 1, maxIndex)
        this.scrollToSelected()
        break
      case 'ArrowUp':
        event.preventDefault()
        this.selectedIndex = Math.max(this.selectedIndex - 1, -1)
        this.scrollToSelected()
        break
      case 'Enter':
        event.preventDefault()
        if (this.selectedIndex >= 0 && this.selectedIndex <= maxIndex) {
          if (isShowingRecent) {
            this.navigateToRecent(this.recentSearches[this.selectedIndex])
          } else {
            this.navigateToResult(this.results[this.selectedIndex])
          }
        }
        break
    }
  }

  /**
   * Scroll the selected result into view
   */
  private scrollToSelected() {
    const resultsList = this.querySelector('.results')
    const selectedItem = this.querySelector(
      `.result[data-index="${this.selectedIndex}"]`
    )
    if (resultsList && selectedItem) {
      selectedItem.scrollIntoView({ block: 'nearest' })
    }
  }

  /**
   * Handle search input
   */
  private handleInput = (event: InputEvent) => {
    const input = event.target as HTMLInputElement
    this.query = input.value

    // Clear previous timeout
    if (this.debounceTimeout) {
      clearTimeout(this.debounceTimeout)
    }

    // Debounce the search
    this.debounceTimeout = window.setTimeout(() => {
      this.performSearch()
    }, this.debounceDelay)
  }

  /**
   * Perform the search
   */
  private async performSearch() {
    if (!this.engine || !this.query.trim()) {
      // Increment version to invalidate any in-flight searches
      this.searchVersion++
      this.results = []
      this.selectedIndex = -1
      return
    }

    // Increment version to track this search request
    const currentVersion = ++this.searchVersion

    this.isSearching = true
    this.error = null

    try {
      const results = await this.engine.search(this.query, {
        limit: this.maxResults,
      })

      // Only update if this is still the latest search
      if (currentVersion !== this.searchVersion) {
        return
      }

      this.results = results
      this.selectedIndex = this.results.length > 0 ? 0 : -1
    } catch (_e) {
      // Only update if this is still the latest search
      if (currentVersion !== this.searchVersion) {
        return
      }

      this.error = 'Search failed. Please try again.'
      this.results = []
      this.selectedIndex = -1
    } finally {
      // Only clear loading state if this is still the latest search
      if (currentVersion === this.searchVersion) {
        this.isSearching = false
      }
    }
  }

  /**
   * CSS class name for highlight animation (defined in site-search.css)
   */
  private static readonly HIGHLIGHT_CLASS = 'search-result-highlight'

  /**
   * Highlight an element briefly to draw attention to it
   */
  private highlightElement(nodeId: string): void {
    // Find the element by ID
    const element = document.getElementById(nodeId)
    if (!element) {
      return
    }

    // Remove any existing highlight first
    element.classList.remove(StencilaSiteSearch.HIGHLIGHT_CLASS)

    // Force a reflow to restart animation if already applied
    void element.offsetWidth

    // Add the highlight class
    element.classList.add(StencilaSiteSearch.HIGHLIGHT_CLASS)

    // Remove the class after animation completes
    element.addEventListener(
      'animationend',
      () => {
        element.classList.remove(StencilaSiteSearch.HIGHLIGHT_CLASS)
      },
      { once: true }
    )
  }

  /**
   * Navigate to a search result
   */
  private async navigateToResult(result: SearchResult) {
    // Save to recent searches before navigating
    this.saveRecentSearch(result.entry)

    // For root nodes (depth 0), navigate without hash to show from top
    // For specific elements, include hash to scroll and highlight
    const isRoot = result.entry.depth === 0
    const url = isRoot
      ? result.entry.route
      : `${result.entry.route}#${result.entry.nodeId}`

    this.close()

    const navigated = await navigate(url, 'programmatic')
    if (navigated && !isRoot) {
      // Delay to let scroll and page settle after navigation
      setTimeout(() => {
        this.highlightElement(result.entry.nodeId)
      }, 100)
    }
  }

  /**
   * Navigate to a recent search
   */
  private async navigateToRecent(recent: RecentSearch) {
    // Move to front of recent searches
    this.saveRecentSearch(recent as SearchEntry)

    // For root nodes (depth 0), navigate without hash to show from top
    // For specific elements, include hash to scroll and highlight
    const isRoot = recent.depth === 0
    const url = isRoot
      ? recent.route
      : `${recent.route}#${recent.nodeId}`

    this.close()

    const navigated = await navigate(url, 'programmatic')
    if (navigated && !isRoot) {
      // Delay to let scroll and page settle after navigation
      setTimeout(() => {
        this.highlightElement(recent.nodeId)
      }, 100)
    }
  }

  /**
   * Open the search modal
   */
  open() {
    this.isOpen = true
    this.query = ''
    this.results = []
    this.error = null

    // Start with no selection - user can arrow down to select
    this.selectedIndex = -1

    // Focus the input after render
    requestAnimationFrame(() => {
      const input = this.querySelector<HTMLInputElement>('.input')
      input?.focus()
    })
  }

  /**
   * Close the search modal
   */
  close() {
    // Increment version to invalidate any in-flight searches
    this.searchVersion++
    this.isOpen = false
    this.query = ''
    this.results = []
    this.selectedIndex = -1
  }

  /**
   * Handle clicking on the backdrop
   */
  private handleBackdropClick = (event: MouseEvent) => {
    if ((event.target as Element).classList.contains('backdrop')) {
      this.close()
    }
  }

  /**
   * Get the icon class for a node type
   */
  private getNodeTypeIcon(nodeType: string): string {
    switch (nodeType) {
      case 'Article':
        return 'i-lucide:file-text'
      case 'Heading':
        return 'i-lucide:heading'
      case 'Paragraph':
        return 'i-lucide:align-left'
      case 'CodeChunk':
        return 'i-lucide:code'
      case 'Datatable':
        return 'i-lucide:table'
      case 'Figure':
        return 'i-lucide:image'
      case 'Table':
        return 'i-lucide:table-2'
      default:
        return 'i-lucide:file'
    }
  }

  /**
   * Render highlighted text with matches
   */
  private renderHighlightedText(text: string, highlights: { start: number; end: number }[]) {
    if (highlights.length === 0) {
      return this.truncateText(text, 150)
    }

    const parts: (string | ReturnType<typeof html>)[] = []
    let lastIndex = 0

    // Sort highlights by start position
    const sortedHighlights = [...highlights].sort((a, b) => a.start - b.start)

    for (const highlight of sortedHighlights) {
      // Add text before highlight
      if (highlight.start > lastIndex) {
        parts.push(text.slice(lastIndex, highlight.start))
      }
      // Add highlighted text
      parts.push(html`<mark class="highlight">${text.slice(highlight.start, highlight.end)}</mark>`)
      lastIndex = highlight.end
    }

    // Add remaining text
    if (lastIndex < text.length) {
      parts.push(text.slice(lastIndex))
    }

    // Truncate if the original text is too long (count code points, not UTF-16 units)
    if ([...text].length > 150) {
      // Find a good truncation point near the first highlight
      const firstHighlight = sortedHighlights[0]
      // Use safe boundaries to avoid splitting surrogate pairs
      const start = this.safeBoundary(text, Math.max(0, firstHighlight.start - 40))
      const end = this.safeBoundary(text, Math.min(text.length, start + 150))

      return html`${start > 0 ? '...' : ''}${this.renderHighlightedTextSlice(text, sortedHighlights, start, end)}${end < text.length ? '...' : ''}`
    }

    return parts
  }

  /**
   * Render a slice of highlighted text
   *
   * Note: sliceStart and sliceEnd should already be safe boundaries
   * (not splitting surrogate pairs). Highlight positions are preserved
   * as-is since they come from the search engine's UTF-16 indexing.
   */
  private renderHighlightedTextSlice(
    text: string,
    highlights: { start: number; end: number }[],
    sliceStart: number,
    sliceEnd: number
  ) {
    const parts: (string | ReturnType<typeof html>)[] = []
    let lastIndex = sliceStart

    for (const highlight of highlights) {
      // Skip highlights outside the slice
      if (highlight.end <= sliceStart || highlight.start >= sliceEnd) {
        continue
      }

      // Clamp highlight to slice bounds, using safe boundaries
      const highlightStart = this.safeBoundary(
        text,
        Math.max(highlight.start, sliceStart)
      )
      const highlightEnd = this.safeBoundary(
        text,
        Math.min(highlight.end, sliceEnd)
      )

      // Add text before highlight
      if (highlightStart > lastIndex) {
        parts.push(text.slice(lastIndex, highlightStart))
      }
      // Add highlighted text
      parts.push(html`<mark class="highlight">${text.slice(highlightStart, highlightEnd)}</mark>`)
      lastIndex = highlightEnd
    }

    // Add remaining text
    if (lastIndex < sliceEnd) {
      parts.push(text.slice(lastIndex, sliceEnd))
    }

    return parts
  }

  /**
   * Truncate text to a maximum length (in code points, not UTF-16 code units)
   *
   * This prevents splitting emoji/astral characters which are represented
   * as surrogate pairs in JavaScript strings.
   */
  private truncateText(text: string, maxCodePoints: number): string {
    const codePoints = [...text]
    if (codePoints.length <= maxCodePoints) {
      return text
    }
    return codePoints.slice(0, maxCodePoints).join('') + '...'
  }

  /**
   * Get a safe slice boundary that doesn't split surrogate pairs
   *
   * If the position is in the middle of a surrogate pair, adjust it.
   */
  private safeBoundary(text: string, pos: number): number {
    if (pos <= 0) return 0
    if (pos >= text.length) return text.length

    // Check if we're at a low surrogate (second half of a pair)
    const code = text.charCodeAt(pos)
    if (code >= 0xdc00 && code <= 0xdfff) {
      // We're at a low surrogate, move back to include the high surrogate
      return pos - 1
    }

    return pos
  }

  /**
   * Load recent searches from localStorage
   */
  private loadRecentSearches(): void {
    try {
      const stored = localStorage.getItem(this.STORAGE_KEY)
      if (stored) {
        this.recentSearches = JSON.parse(stored)
      }
    } catch {
      // Ignore localStorage errors (e.g., private browsing)
      this.recentSearches = []
    }
  }

  /**
   * Save a search entry to recent searches
   */
  private saveRecentSearch(entry: SearchEntry): void {
    // Create a minimal recent search object
    const recent: RecentSearch = {
      nodeId: entry.nodeId,
      nodeType: entry.nodeType,
      route: entry.route,
      breadcrumbs: entry.breadcrumbs,
      text: entry.text,
      depth: entry.depth,
    }

    // Remove any existing entry with the same route + nodeId (to move it to front)
    // Node IDs are document-scoped, so we need both to uniquely identify an entry
    const filtered = this.recentSearches.filter(
      (r) => r.route !== recent.route || r.nodeId !== recent.nodeId
    )

    // Add to front and limit to MAX_RECENT
    this.recentSearches = [recent, ...filtered].slice(0, this.MAX_RECENT)

    // Persist to localStorage
    try {
      localStorage.setItem(this.STORAGE_KEY, JSON.stringify(this.recentSearches))
    } catch {
      // Ignore localStorage errors
    }
  }

  /**
   * Format breadcrumbs for display
   */
  private formatBreadcrumbs(breadcrumbs: string[] | undefined): string {
    // Handle missing breadcrumbs (e.g., from old cached search index)
    if (!breadcrumbs || breadcrumbs.length === 0) {
      return 'Home'
    }
    // Join breadcrumb labels with chevron separator
    return breadcrumbs.join(' > ')
  }

  /**
   * Render a single result item (reusable for results and recent searches)
   */
  private renderResultItem(
    entry: {
      nodeId: string
      nodeType: string
      route: string
      breadcrumbs?: string[]
      text: string
    },
    index: number,
    highlights: { start: number; end: number }[],
    onClick: () => void
  ) {
    return html`
      <li
        class="result ${index === this.selectedIndex ? 'selected' : ''}"
        data-index=${index}
        role="option"
        aria-selected=${index === this.selectedIndex}
        @click=${onClick}
        @mouseenter=${() => {
          this.selectedIndex = index
        }}
      >
        <span class="${this.getNodeTypeIcon(entry.nodeType)} icon"></span>
        <div class="content">
          <div class="path">${this.formatBreadcrumbs(entry.breadcrumbs)}</div>
          <div class="text">
            ${this.renderHighlightedText(entry.text, highlights)}
          </div>
        </div>
      </li>
    `
  }

  /**
   * Render the recent searches section
   */
  private renderRecentSearches() {
    if (this.recentSearches.length === 0) {
      return nothing
    }

    return html`
      <div class="recent-header">Recent searches</div>
      <ul class="results" role="listbox">
        ${this.recentSearches.map((recent, index) =>
          this.renderResultItem(
            recent,
            index,
            [], // No highlights for recent searches
            () => this.navigateToRecent(recent)
          )
        )}
      </ul>
    `
  }

  protected override render() {
    // Render the trigger button
    const trigger = html`
      <button
        class="trigger"
        @click=${this.open}
        aria-label="Open search"
        title="Search (Cmd+K)"
      >
        <span class="i-lucide:search icon"></span>
        <span class="label">${this.placeholder}</span>
        <kbd class="shortcut">
          <span class="key">⌘</span>
          <span class="key">K</span>
        </kbd>
      </button>
    `

    // Render the modal if open
    const modal = this.isOpen
      ? html`
          <div class="backdrop" @click=${this.handleBackdropClick}>
            <div class="modal" role="dialog" aria-modal="true" aria-label="Search">
              <div class="header">
                <span class="i-lucide:search icon"></span>
                <input
                  type="text"
                  class="input"
                  .value=${this.query}
                  @input=${this.handleInput}
                  @keydown=${this.handleInputKeydown}
                  placeholder=${this.placeholder}
                  aria-label="Search query"
                  autocomplete="off"
                  autocorrect="off"
                  autocapitalize="off"
                  spellcheck="false"
                />
                ${this.isSearching
                  ? html`<span class="i-lucide:loader-2 loading"></span>`
                  : nothing}
                <button
                  class="close"
                  @click=${this.close}
                  aria-label="Close search"
                >
                  <kbd>esc</kbd>
                </button>
              </div>

              ${this.error
                ? html`<div class="error">${this.error}</div>`
                : nothing}

              ${this.results.length > 0
                ? html`
                    <ul class="results" role="listbox">
                      ${this.results.map((result, index) =>
                        this.renderResultItem(
                          result.entry,
                          index,
                          result.highlights,
                          () => this.navigateToResult(result)
                        )
                      )}
                    </ul>
                  `
                : this.query && !this.isSearching
                  ? html`<div class="empty">No results found</div>`
                  : !this.query
                    ? this.renderRecentSearches()
                    : nothing}

              <div class="footer">
                <span class="hint">
                  <kbd>↑</kbd><kbd>↓</kbd> to navigate
                </span>
                <span class="hint">
                  <kbd>↵</kbd> to select
                </span>
                <span class="hint">
                  <kbd>esc</kbd> to close
                </span>
              </div>
            </div>
          </div>
        `
      : nothing

    return html`${trigger}${modal}`
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'stencila-site-search': StencilaSiteSearch
  }
}
