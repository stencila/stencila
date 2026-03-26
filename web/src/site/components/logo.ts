import { LitElement } from 'lit'
import { customElement, property } from 'lit/decorators'

const MOBILE_QUERY = '(max-width: 639px)'
const TABLET_QUERY = '(min-width: 640px) and (max-width: 767px)'
const DARK_PREF_QUERY = '(prefers-color-scheme: dark)'

/**
 * Logo component
 *
 * Works with the native `<picture>` element rendered by the server.
 * `<picture>` handles system dark mode preference and viewport breakpoints
 * without JS via `<source media="...">` elements.
 *
 * This component intervenes when the user has explicitly overridden the
 * system color scheme via `data-color-scheme` (set by the color-mode toggle).
 * In that case `<picture>` media queries for `prefers-color-scheme` no longer
 * reflect the user's intent, so we disable all `<source>` elements (by setting
 * `media="not all"`) and set `img.src` directly to the correct variant.
 *
 * When the explicit scheme matches the system preference (or is absent), we
 * restore the original `<source>` media attributes and `img.src` so `<picture>`
 * resumes native behavior.
 */
@customElement('stencila-logo')
export class StencilaLogo extends LitElement {
  @property({ type: String })
  default: string | null = null

  @property({ type: String })
  mobile: string | null = null

  @property({ type: String })
  tablet: string | null = null

  @property({ type: String })
  dark: string | null = null

  @property({ type: String, attribute: 'dark-mobile' })
  darkMobile: string | null = null

  @property({ type: String, attribute: 'dark-tablet' })
  darkTablet: string | null = null

  /**
   * Use Light DOM - the server renders the content, we just update it
   */
  protected override createRenderRoot() {
    return this
  }

  private mediaQueries: MediaQueryList[] = []
  private observer: MutationObserver | undefined

  /**
   * The original `src` from the server-rendered `<img>`, used to restore
   * native `<picture>` behavior when no explicit override is active.
   */
  private originalSrc: string | null = null

  /**
   * Original media attributes from `<source>` elements, keyed by index.
   * Saved on connect so we can restore them after disabling.
   */
  private originalSourceMedia: string[] = []

  override connectedCallback() {
    super.connectedCallback()

    const img = this.querySelector('img')
    if (img) {
      this.originalSrc = img.getAttribute('src')
    }

    // Save original media attributes from <source> elements
    const sources = this.querySelectorAll('source')
    this.originalSourceMedia = Array.from(sources).map(
      (s) => s.getAttribute('media') ?? ''
    )

    this.applyLogo()

    // Watch for viewport breakpoint changes and system dark mode preference
    this.mediaQueries = [MOBILE_QUERY, TABLET_QUERY, DARK_PREF_QUERY].map(
      (q) => matchMedia(q)
    )
    for (const mql of this.mediaQueries) {
      mql.addEventListener('change', this.applyLogo)
    }

    // Watch for user toggling color scheme
    this.observer = new MutationObserver(this.applyLogo)
    this.observer.observe(document.documentElement, {
      attributes: true,
      attributeFilter: ['data-color-scheme'],
    })
  }

  override disconnectedCallback() {
    super.disconnectedCallback()

    for (const mql of this.mediaQueries) {
      mql.removeEventListener('change', this.applyLogo)
    }
    this.mediaQueries = []

    this.observer?.disconnect()
  }

  override updated() {
    this.applyLogo()
  }

  private applyLogo = () => {
    const img = this.querySelector('img') as HTMLImageElement | null
    if (!img) return

    const sources = this.querySelectorAll('source')
    const root = document.documentElement
    const colorScheme = root.getAttribute('data-color-scheme')
    const systemPrefersDark = matchMedia(DARK_PREF_QUERY).matches
    const systemScheme = systemPrefersDark ? 'dark' : 'light'

    // When the explicit scheme matches the system preference (or is absent),
    // let <picture> handle everything natively.
    if (!colorScheme || colorScheme === systemScheme) {
      // Restore original <source> media attributes
      sources.forEach((source, i) => {
        const original = this.originalSourceMedia[i]
        if (original !== undefined) {
          source.setAttribute('media', original)
        }
      })

      // Restore original img src so <picture> source selection takes effect
      if (this.originalSrc !== null) {
        img.src = this.originalSrc
      }
      return
    }

    // User has explicitly overridden the system preference.
    // Disable all <source> elements so the browser cannot select them,
    // then set img.src directly to the correct variant.
    sources.forEach((source) => {
      source.setAttribute('media', 'not all')
    })

    const url = this.getVariantUrl(colorScheme === 'dark')
    if (url) {
      img.src = url
    }
  }

  private getVariantUrl(isDark: boolean): string | null {
    const isMobile = matchMedia(MOBILE_QUERY).matches
    const isTablet = matchMedia(TABLET_QUERY).matches

    if (isDark && isMobile) {
      return this.darkMobile ?? this.dark ?? this.mobile ?? this.default
    }
    if (isDark && isTablet) {
      return this.darkTablet ?? this.dark ?? this.tablet ?? this.default
    }
    if (isDark) {
      return this.dark ?? this.default
    }
    if (isMobile) {
      return this.mobile ?? this.default
    }
    if (isTablet) {
      return this.tablet ?? this.default
    }
    return this.default
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'stencila-logo': StencilaLogo
  }
}
