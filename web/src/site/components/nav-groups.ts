import { LitElement } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { GlideEvents } from '../glide/events'
import {
  addAccessBadgeToItem,
  canAccess,
  getAccessConfig,
} from '../utils/access'

import type {
  AccessLevel,
  RouteAccessConfig,
  SiteAuthStatusResponse,
} from './site-action/types'
import { getCachedAuthStatus, isLocalhost } from './site-action/utils'

/**
 * Navigation groups component
 *
 * Provides SPA navigation support for footer-style grouped links.
 * Updates active state when navigating between pages.
 *
 * The HTML structure is rendered server-side by Rust. This component
 * adds client-side interactivity for SPA navigation.
 */
@customElement('stencila-nav-groups')
export class StencilaNavGroups extends LitElement {
  /**
   * How to handle items requiring higher access than user has
   * - 'hide': Hide items user cannot access (default)
   * - 'show': Show all items (restricted items remain visible)
   */
  @property({ type: String, attribute: 'restricted-items' })
  restrictedItems: 'hide' | 'show' = 'hide'

  /**
   * Whether to show access badges on items that are restricted
   * Badges indicate the required access level (password, team, etc.)
   */
  @property({ type: Boolean, attribute: 'show-access-badges' })
  showAccessBadges = true

  /**
   * Access configuration (from _access.json)
   */
  @state()
  private accessConfig: RouteAccessConfig | undefined = undefined

  /**
   * User's access level from auth status
   */
  @state()
  private userAccessLevel: AccessLevel = 'public'

  /**
   * Use Light DOM so theme CSS can style the component
   */
  protected override createRenderRoot() {
    return this
  }

  override connectedCallback() {
    super.connectedCallback()

    // Load access configuration and apply filtering
    this.loadAccessAndFilter()

    // Listen for SPA navigation to update active state
    window.addEventListener(GlideEvents.END, this.handleGlideEnd)
  }

  override disconnectedCallback() {
    super.disconnectedCallback()
    window.removeEventListener(GlideEvents.END, this.handleGlideEnd)
  }

  /**
   * Load access configuration and user access level, then apply filtering
   */
  private async loadAccessAndFilter() {
    try {
      // On localhost preview, grant team access (user has repo access)
      if (isLocalhost()) {
        this.userAccessLevel = 'team'
        // Still fetch access config to show badges
        this.accessConfig = await getAccessConfig()
        if (this.accessConfig) {
          this.applyAccessFiltering()
        }
        return
      }

      // Fetch access config and auth status in parallel
      const [config, authStatus] = await Promise.all([
        getAccessConfig(),
        this.fetchAuthStatus(),
      ])

      this.accessConfig = config
      this.userAccessLevel = authStatus?.userAccessLevel ?? 'public'

      // Apply access filtering if we have restrictions
      if (config) {
        this.applyAccessFiltering()
      }
    } catch (error) {
      console.warn('[StencilaNavGroups] Failed to load access config:', error)
    }
  }

  /**
   * Fetch auth status to get user's access level
   */
  private async fetchAuthStatus(): Promise<SiteAuthStatusResponse | null> {
    try {
      // Use the cached auth status fetcher
      return await getCachedAuthStatus(
        '',
        '/__stencila/auth/status',
        async () => ({})
      )
    } catch {
      return null
    }
  }

  /**
   * Apply access-based filtering to nav items
   *
   * Hides items the user cannot access (if restrictedItems='hide')
   * and adds badges to restricted items (if showAccessBadges=true).
   */
  private applyAccessFiltering() {
    if (!this.accessConfig) {
      return
    }

    // Filter link items (.link with data-access)
    const linkItems = this.querySelectorAll<HTMLElement>('.link[data-access]')

    for (const item of linkItems) {
      const requiredLevel = item.dataset.access as AccessLevel
      const userCanAccess = canAccess(this.userAccessLevel, requiredLevel)

      if (!userCanAccess && this.restrictedItems === 'hide') {
        // Hide items user cannot access
        item.style.display = 'none'
      } else if (this.showAccessBadges) {
        // Add badge to indicate access restriction
        addAccessBadgeToItem(item, requiredLevel, 'a .label')
      }
    }

    // Filter group headings (.group-heading with data-access)
    const headingItems = this.querySelectorAll<HTMLElement>(
      '.group-heading[data-access]'
    )

    for (const item of headingItems) {
      const requiredLevel = item.dataset.access as AccessLevel
      const userCanAccess = canAccess(this.userAccessLevel, requiredLevel)

      if (!userCanAccess && this.restrictedItems === 'hide') {
        // Hide the entire group when heading is restricted
        const group = item.closest('.group')
        if (group) {
          ;(group as HTMLElement).style.display = 'none'
        }
      } else if (this.showAccessBadges) {
        // Add badge to group heading
        addAccessBadgeToItem(item, requiredLevel, 'a')
      }
    }
  }

  /**
   * Handle SPA navigation end event
   */
  private handleGlideEnd = () => {
    // Small delay to ensure DOM is updated
    requestAnimationFrame(() => {
      this.updateActiveState()
    })
  }

  /**
   * Update the active state based on current URL
   */
  private updateActiveState() {
    const currentPath = window.location.pathname

    // Remove active state from all link items
    const activeLinks = this.querySelectorAll('.link[data-active="true"]')
    for (const link of activeLinks) {
      link.setAttribute('data-active', 'false')
      const anchor = link.querySelector('a')
      if (anchor) {
        anchor.removeAttribute('aria-current')
      }
    }

    // Remove aria-current from all group heading links
    const headingLinks = this.querySelectorAll<HTMLAnchorElement>(
      '.group-heading a[aria-current]'
    )
    for (const anchor of headingLinks) {
      anchor.removeAttribute('aria-current')
    }

    // Find and activate matching links (both .link items and group headings)
    const allAnchors = this.querySelectorAll<HTMLAnchorElement>(
      '.link a[href], .group-heading a[href]'
    )
    for (const anchor of allAnchors) {
      const href = anchor.getAttribute('href')
      // Match exact path or path with trailing slash
      if (href === currentPath || href === currentPath + '/') {
        anchor.setAttribute('aria-current', 'page')

        // Also set data-active on .link items
        const linkItem = anchor.closest('.link')
        if (linkItem) {
          linkItem.setAttribute('data-active', 'true')
        }
      }
    }
  }
}
