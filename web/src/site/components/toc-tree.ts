import { LitElement, PropertyValues, html, nothing } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { GlideEvents } from '../glide/events'

import type { StencilaLayout } from './layout'

/**
 * Represents a heading in the table of contents
 */
interface TocHeading {
  id: string
  text: string
  level: number
  element: HTMLElement
}

/**
 * Table of Contents tree component
 *
 * Dynamically extracts headings from the document and provides:
 * - Hierarchical navigation structure
 * - Scroll-spy highlighting of active section
 * - Smooth scrolling to sections
 * - Keyboard navigation
 * - SPA navigation support
 */
@customElement('stencila-toc-tree')
export class StencilaTocTree extends LitElement {
  /**
   * Title displayed above the TOC (default: "On this page")
   * Uses tocTitle to avoid conflict with HTMLElement.title
   */
  @property({ type: String, attribute: 'title' })
  tocTitle = 'On this page'

  /**
   * Maximum heading depth to include (default: 3 = h1-h3)
   */
  @property({ type: Number })
  depth = 3

  /**
   * Extracted headings from the document
   */
  @state()
  private headings: TocHeading[] = []

  /**
   * ID of the currently active heading
   */
  @state()
  private activeId: string | null = null

  /**
   * Cached header offset for IntersectionObserver rootMargin.
   * Updated at scroll-spy setup from CSS variable.
   */
  private headerOffset = 80

  /**
   * IntersectionObserver for scroll-spy
   */
  private observer: IntersectionObserver | null = null

  /**
   * Set of currently intersecting heading IDs.
   * Maintained across observer callbacks to correctly determine the topmost visible heading.
   */
  private intersectingIds: Set<string> = new Set()

  /**
   * Whether scroll-spy is temporarily disabled (during click navigation)
   */
  private scrollSpyDisabled = false

  /**
   * AbortController for scroll end detection
   */
  private scrollEndController: AbortController | null = null

  /**
   * Fallback timeout ID for re-enabling scroll-spy
   */
  private scrollSpyTimeout: ReturnType<typeof setTimeout> | null = null

  /**
   * Use Light DOM so theme CSS can style the component
   */
  protected override createRenderRoot() {
    return this
  }

  override connectedCallback() {
    super.connectedCallback()
    this.extractHeadings()
    this.setupScrollSpy()

    // Listen for SPA navigation to rebuild TOC
    window.addEventListener(GlideEvents.END, this.handleGlideEnd)
  }

  override disconnectedCallback() {
    super.disconnectedCallback()
    this.cleanupScrollSpy()
    window.removeEventListener(GlideEvents.END, this.handleGlideEnd)
    this.cleanupScrollEndListeners()
  }

  override updated(changedProperties: PropertyValues) {
    super.updated(changedProperties)

    // Re-extract headings if depth changes
    if (changedProperties.has('depth')) {
      this.extractHeadings()
      this.setupScrollSpy()
    }
  }

  /**
   * Handle Glide navigation end - reinitialize TOC
   */
  private handleGlideEnd = () => {
    // Small delay to ensure DOM is updated
    requestAnimationFrame(() => {
      this.extractHeadings()
      this.setupScrollSpy()
    })
  }

  /**
   * Extract headings from the main content area
   *
   * Stencila documents use <stencila-heading level="N" id="..."> elements,
   * not standard <h1>, <h2>, etc.
   */
  private extractHeadings() {
    const main = document.querySelector('main')
    if (!main) {
      this.headings = []
      return
    }

    // Query stencila-heading elements with level and id attributes
    const headingElements = main.querySelectorAll<HTMLElement>('stencila-heading[level][id]')

    this.headings = Array.from(headingElements)
      .filter((el) => {
        const level = parseInt(el.getAttribute('level') ?? '0', 10)
        return level > 0 && level <= this.depth
      })
      .map((el) => ({
        id: el.id,
        text: el.textContent?.trim() ?? '',
        level: parseInt(el.getAttribute('level') ?? '1', 10),
        element: el,
      }))
  }

  /**
   * Set up IntersectionObserver for scroll-spy
   */
  private setupScrollSpy() {
    this.cleanupScrollSpy()

    if (this.headings.length === 0) {
      return
    }

    // Get header offset from CSS variable for IntersectionObserver rootMargin
    const headerHeight = getComputedStyle(document.documentElement)
      .getPropertyValue('--layout-header-height')
      .trim()
    this.headerOffset = headerHeight ? parseFloat(headerHeight) || 80 : 80

    // rootMargin: trigger when heading is near top of viewport
    // Header offset for fixed header, -80% bottom to only track top portion
    this.observer = new IntersectionObserver(
      (entries) => {
        if (this.scrollSpyDisabled) {
          return
        }

        // Update the set of currently intersecting headings
        for (const entry of entries) {
          if (entry.isIntersecting) {
            this.intersectingIds.add(entry.target.id)
          } else {
            this.intersectingIds.delete(entry.target.id)
          }
        }

        // Find the topmost intersecting heading by checking headings in DOM order
        for (const heading of this.headings) {
          if (this.intersectingIds.has(heading.id)) {
            this.setActiveId(heading.id)
            return
          }
        }

        // No headings in zone - fall back to scroll position check
        if (this.intersectingIds.size === 0) {
          this.updateActiveFromScroll()
        }
      },
      {
        rootMargin: `-${this.headerOffset}px 0px -80% 0px`,
        threshold: 0,
      }
    )

    // Observe all heading elements
    for (const heading of this.headings) {
      this.observer.observe(heading.element)
    }

    // Set initial active heading based on current scroll position
    // Don't auto-scroll TOC on initial setup - let the page scroll restore naturally
    this.updateActiveFromScroll(false)
  }

  /**
   * Clean up the IntersectionObserver
   */
  private cleanupScrollSpy() {
    if (this.observer) {
      this.observer.disconnect()
      this.observer = null
    }
    this.intersectingIds.clear()
  }

  /**
   * Clean up scroll end listeners and timeout
   */
  private cleanupScrollEndListeners() {
    if (this.scrollEndController) {
      this.scrollEndController.abort()
      this.scrollEndController = null
    }
    if (this.scrollSpyTimeout) {
      clearTimeout(this.scrollSpyTimeout)
      this.scrollSpyTimeout = null
    }
  }

  /**
   * Update active heading based on current scroll position
   *
   * @param scrollIntoView - Whether to scroll the TOC to show the active link (default true)
   */
  private updateActiveFromScroll(scrollIntoView: boolean = true) {
    // Find the heading closest to the top of the viewport
    let activeHeading: TocHeading | null = null

    for (const heading of this.headings) {
      const rect = heading.element.getBoundingClientRect()
      if (rect.top <= this.headerOffset) {
        activeHeading = heading
      } else {
        break
      }
    }

    if (activeHeading) {
      this.setActiveId(activeHeading.id, scrollIntoView)
    } else if (this.headings.length > 0) {
      // Default to first heading if none are above the fold
      this.setActiveId(this.headings[0].id, scrollIntoView)
    }
  }

  /**
   * Set the active heading ID and optionally scroll the TOC to show it
   *
   * @param id - The heading ID to set as active
   * @param scrollIntoView - Whether to scroll the TOC to show the active link
   */
  private setActiveId(id: string, scrollIntoView: boolean = true) {
    if (this.activeId === id) {
      return
    }

    this.activeId = id

    if (scrollIntoView) {
      // Wait for Lit to update the DOM before scrolling
      this.updateComplete.then(() => {
        this.scrollActiveIntoView()
      })
    }
  }

  /**
   * Scroll the TOC container to make the active link visible
   */
  private scrollActiveIntoView() {
    const activeLink = this.querySelector<HTMLAnchorElement>(`a[href="#${this.activeId}"]`)
    if (!activeLink) {
      return
    }

    // Find the scrollable container (the nav element or this component)
    const container = this.closest('nav[slot="right-sidebar"]') || this

    // Check if link is outside the visible area of the container
    const containerRect = container.getBoundingClientRect()
    const linkRect = activeLink.getBoundingClientRect()

    if (linkRect.top < containerRect.top || linkRect.bottom > containerRect.bottom) {
      activeLink.scrollIntoView({ block: 'nearest', behavior: 'smooth' })
    }
  }

  /**
   * Handle click on a TOC link
   */
  private handleLinkClick(event: MouseEvent, heading: TocHeading) {
    event.preventDefault()

    // Clean up any existing scroll end listeners
    this.cleanupScrollEndListeners()

    // Temporarily disable scroll-spy to prevent flickering
    this.scrollSpyDisabled = true

    // Set active state immediately for responsive feel (no TOC scroll needed)
    this.activeId = heading.id

    // Focus the clicked link for keyboard navigation continuity
    const link = event.currentTarget as HTMLAnchorElement
    link.focus()

    // Scroll to the heading (scroll-padding-top handles header offset)
    heading.element.scrollIntoView({ behavior: 'smooth', block: 'start' })

    // Update URL hash without jumping
    history.pushState(null, '', `#${heading.id}`)

    // Close mobile sidebar after navigation
    const layout = this.closest('stencila-layout') as StencilaLayout | null
    layout?.closeSidebars()

    // Re-enable scroll-spy when scroll ends
    this.scrollEndController = new AbortController()
    window.addEventListener(
      'scrollend',
      () => {
        this.reEnableScrollSpy()
      },
      { once: true, signal: this.scrollEndController.signal }
    )

    // Fallback timeout for browsers without scrollend or very long scrolls
    this.scrollSpyTimeout = setTimeout(() => {
      this.reEnableScrollSpy()
    }, 1000)
  }

  /**
   * Re-enable scroll-spy after click navigation completes.
   * Clears stale intersection state but keeps the clicked heading active.
   */
  private reEnableScrollSpy() {
    this.cleanupScrollEndListeners()
    this.scrollSpyDisabled = false
    // Clear stale intersection state accumulated during disabled period
    this.intersectingIds.clear()
    // Don't recalculate active heading - keep the clicked heading active.
    // The IntersectionObserver will update it naturally when the user scrolls.
  }

  /**
   * Handle keyboard navigation within the TOC
   */
  private handleKeydown = (event: KeyboardEvent) => {
    const links = Array.from(this.querySelectorAll<HTMLAnchorElement>('a[href^="#"]'))
    const currentIndex = links.findIndex((link) => link === document.activeElement)

    if (currentIndex === -1) {
      return
    }

    let newIndex: number | null

    switch (event.key) {
      case 'ArrowDown':
        newIndex = Math.min(currentIndex + 1, links.length - 1)
        break
      case 'ArrowUp':
        newIndex = Math.max(currentIndex - 1, 0)
        break
      case 'Home':
        newIndex = 0
        break
      case 'End':
        newIndex = links.length - 1
        break
      default:
        return
    }

    if (newIndex !== null && newIndex !== currentIndex) {
      event.preventDefault()
      links[newIndex].focus()
    }
  }

  /**
   * Reinitialize the TOC (called after SPA navigation)
   * This method is called by glide.ts after page transitions
   */
  reinitialize() {
    this.extractHeadings()
    this.setupScrollSpy()
  }

  protected override render() {
    if (this.headings.length === 0) {
      return nothing
    }

    return html`
      <nav aria-label="Table of contents" @keydown=${this.handleKeydown}>
        <h2 class="toc-title">${this.tocTitle}</h2>
        <ul class="toc-list" role="list">
          ${this.headings.map((heading) => this.renderHeading(heading))}
        </ul>
      </nav>
    `
  }

  /**
   * Render a single heading item
   */
  private renderHeading(heading: TocHeading) {
    const isActive = this.activeId === heading.id

    return html`
      <li
        class="toc-item"
        data-level="${heading.level}"
        data-active="${isActive ? 'true' : 'false'}"
      >
        <a
          href="#${heading.id}"
          @click=${(e: MouseEvent) => this.handleLinkClick(e, heading)}
          .ariaCurrent=${isActive ? 'location' : null}
        >
          ${heading.text}
        </a>
      </li>
    `
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'stencila-toc-tree': StencilaTocTree
  }
}
