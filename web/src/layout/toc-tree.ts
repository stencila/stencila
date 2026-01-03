import { LitElement } from 'lit'
import { customElement } from 'lit/decorators.js'

/**
 * Table of contents tree component for right sidebar
 *
 * A Light DOM component that enhances the server-rendered TOC with:
 * - IntersectionObserver-based active heading tracking
 * - Smooth scroll behavior
 * - Keyboard navigation
 *
 * The HTML is server-rendered with ARIA tree pattern:
 * - ul[role="tree"] - root TOC list
 * - li[role="treeitem"] - TOC items
 * - a.toc-link - links to heading anchors
 * - ul[role="group"] - nested groups
 *
 * Active tracking uses IntersectionObserver to detect which heading
 * is currently in the top portion of the viewport, updating the
 * corresponding TOC link with a `data-active` attribute.
 */
@customElement('stencila-toc-tree')
export class StencilaTocTree extends LitElement {
  /** IntersectionObserver for tracking visible headings */
  private observer: IntersectionObserver | null = null

  /** Map of heading IDs to their TOC link elements */
  private tocLinks: Map<string, HTMLAnchorElement> = new Map()

  /** Currently active heading ID */
  private activeId: string | null = null

  /** Heading elements being observed */
  private headingElements: HTMLElement[] = []

  /** Flag to temporarily ignore observer updates during programmatic scroll */
  private isScrolling: boolean = false

  /**
   * Override to use Light DOM so theme CSS applies
   */
  protected override createRenderRoot() {
    return this
  }

  override connectedCallback() {
    super.connectedCallback()

    // Build map of heading IDs to TOC links
    this.buildTocLinkMap()

    // Set up IntersectionObserver for active heading tracking
    this.setupObserver()

    // Set up smooth scroll behavior
    this.setupSmoothScroll()

    // Set up keyboard navigation
    this.addEventListener('keydown', this.handleKeydown)
  }

  override disconnectedCallback() {
    super.disconnectedCallback()

    // Clean up observer
    if (this.observer) {
      this.observer.disconnect()
      this.observer = null
    }

    this.removeEventListener('keydown', this.handleKeydown)
  }

  /**
   * Build a map of heading IDs to their corresponding TOC link elements
   */
  private buildTocLinkMap() {
    this.tocLinks.clear()
    const links = this.querySelectorAll<HTMLAnchorElement>('a.toc-link[href^="#"]')

    for (const link of links) {
      const href = link.getAttribute('href')
      if (href) {
        // Extract ID from href (remove the leading #)
        const id = href.slice(1)
        this.tocLinks.set(id, link)
      }
    }
  }

  /**
   * Set up IntersectionObserver to track which heading is in view
   *
   * Uses a root margin that considers the top 20% of the viewport as
   * the "active zone". When a heading enters this zone, it becomes active.
   */
  private setupObserver() {
    // Find all heading elements referenced by TOC links
    this.headingElements = []
    for (const id of this.tocLinks.keys()) {
      const heading = document.getElementById(id)
      if (heading) {
        this.headingElements.push(heading)
      }
    }

    if (this.headingElements.length === 0) {
      return
    }

    // Create observer with margins that define the "active zone"
    // -80px top margin accounts for fixed headers
    // -80% bottom margin means only the top 20% of viewport triggers
    this.observer = new IntersectionObserver(
      this.handleIntersection.bind(this),
      {
        rootMargin: '-80px 0px -80% 0px',
        threshold: 0,
      }
    )

    // Observe all heading elements
    for (const heading of this.headingElements) {
      this.observer.observe(heading)
    }

    // Set initial active state based on current scroll position
    this.updateActiveFromScroll()
  }

  /**
   * Handle intersection events from the observer
   */
  private handleIntersection(entries: IntersectionObserverEntry[]) {
    // Skip updates during programmatic scrolling to prevent flicker
    if (this.isScrolling) {
      return
    }

    // Find entries that are intersecting (visible in the active zone)
    const intersecting = entries.filter((entry) => entry.isIntersecting)

    if (intersecting.length > 0) {
      // If multiple headings are visible, pick the one closest to the top
      const sorted = intersecting.sort(
        (a, b) => a.boundingClientRect.top - b.boundingClientRect.top
      )
      const topmost = sorted[0]
      this.setActiveHeading(topmost.target.id)
    } else {
      // No headings in active zone - find the heading just above the viewport
      this.updateActiveFromScroll()
    }
  }

  /**
   * Update active heading based on scroll position
   *
   * Used when no heading is in the intersection zone (e.g., scrolled
   * between headings). Finds the last heading that's above the viewport.
   */
  private updateActiveFromScroll() {
    const scrollTop = window.scrollY + 100 // Account for header offset

    // Find the last heading that's above the current scroll position
    let lastAbove: HTMLElement | null = null
    for (const heading of this.headingElements) {
      const rect = heading.getBoundingClientRect()
      const absoluteTop = rect.top + window.scrollY

      if (absoluteTop <= scrollTop) {
        lastAbove = heading
      } else {
        break // Headings are in DOM order, so we can stop here
      }
    }

    if (lastAbove) {
      this.setActiveHeading(lastAbove.id)
    } else if (this.headingElements.length > 0) {
      // If we're above all headings, activate the first one
      this.setActiveHeading(this.headingElements[0].id)
    }
  }

  /**
   * Set the active heading and update TOC link styles
   */
  private setActiveHeading(id: string) {
    if (this.activeId === id) {
      return // No change
    }

    // Remove active state from previous link
    if (this.activeId) {
      const prevLink = this.tocLinks.get(this.activeId)
      if (prevLink) {
        prevLink.removeAttribute('data-active')
        prevLink.removeAttribute('aria-current')
      }
    }

    // Set active state on new link
    const newLink = this.tocLinks.get(id)
    if (newLink) {
      newLink.setAttribute('data-active', 'true')
      newLink.setAttribute('aria-current', 'location')

      // Ensure the active link is visible in the TOC (scroll if needed)
      this.scrollActiveIntoView(newLink)
    }

    this.activeId = id
  }

  /**
   * Scroll the TOC container to make the active link visible
   */
  private scrollActiveIntoView(link: HTMLAnchorElement) {
    // Find the scrollable container (the nav element or this component)
    const container = this.closest('nav[slot="right-sidebar"]') || this

    // Check if link is outside the visible area of the container
    const containerRect = container.getBoundingClientRect()
    const linkRect = link.getBoundingClientRect()

    if (linkRect.top < containerRect.top || linkRect.bottom > containerRect.bottom) {
      link.scrollIntoView({ block: 'nearest', behavior: 'smooth' })
    }
  }

  /**
   * Set up smooth scroll behavior for TOC links
   */
  private setupSmoothScroll() {
    this.addEventListener('click', (event) => {
      const target = event.target as HTMLElement
      const link = target.closest('a.toc-link') as HTMLAnchorElement | null

      if (!link) return

      const href = link.getAttribute('href')
      if (!href?.startsWith('#')) return

      const id = href.slice(1)
      const heading = document.getElementById(id)

      if (heading) {
        event.preventDefault()

        // Set active state immediately for responsive feel
        this.setActiveHeading(id)

        // Ensure the active link has focus for keyboard nav continuity
        // This also removes focus from any previously focused link
        const activeLink = this.tocLinks.get(id)
        if (activeLink) {
          activeLink.focus()
        }

        // Disable observer updates during scroll to prevent flicker
        this.isScrolling = true

        // Scroll to heading with offset for fixed header
        const headerOffset = 80
        const elementPosition = heading.getBoundingClientRect().top
        const offsetPosition = elementPosition + window.scrollY - headerOffset

        window.scrollTo({
          top: offsetPosition,
          behavior: 'smooth',
        })

        // Update URL hash without jumping
        history.pushState(null, '', href)

        // Re-enable observer after scroll animation completes
        setTimeout(() => {
          this.isScrolling = false
        }, 500)
      }
    })
  }

  /**
   * Handle keyboard navigation
   */
  private handleKeydown = (event: KeyboardEvent) => {
    const target = event.target as HTMLElement
    if (!target?.matches('a.toc-link')) return

    const links = Array.from(this.querySelectorAll<HTMLAnchorElement>('a.toc-link'))
    const currentIndex = links.indexOf(target as HTMLAnchorElement)

    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault()
        if (currentIndex < links.length - 1) {
          links[currentIndex + 1].focus()
        }
        break

      case 'ArrowUp':
        event.preventDefault()
        if (currentIndex > 0) {
          links[currentIndex - 1].focus()
        }
        break

      case 'Home':
        event.preventDefault()
        links[0]?.focus()
        break

      case 'End':
        event.preventDefault()
        links[links.length - 1]?.focus()
        break
    }
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'stencila-toc-tree': StencilaTocTree
  }
}
