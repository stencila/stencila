import { LitElement } from 'lit'
import { customElement } from 'lit/decorators.js'

/**
 * Map of icon names to UnoCSS icon classes
 *
 * Uses simple-icons for brand logos and lucide for utility icons.
 */
const ICON_CLASSES: Record<string, string> = {
  // Brand icons (simple-icons)
  github: 'i-simple-icons:github',
  discord: 'i-simple-icons:discord',
  twitter: 'i-simple-icons:x',
  x: 'i-simple-icons:x',
  linkedin: 'i-simple-icons:linkedin',
  youtube: 'i-simple-icons:youtube',
  bluesky: 'i-simple-icons:bluesky',
  mastodon: 'i-simple-icons:mastodon',
  slack: 'i-simple-icons:slack',

  // Utility icons (lucide)
  rss: 'i-lucide:rss',
  mail: 'i-lucide:mail',
  'external-link': 'i-lucide:external-link',
}

/**
 * Site footer component
 *
 * A Light DOM component that enhances the server-rendered footer with:
 * - Icon rendering (replaces placeholder spans with UnoCSS icon classes)
 *
 * The HTML is server-rendered for SEO and fast paint.
 */
@customElement('stencila-footer')
export class StencilaFooter extends LitElement {
  /**
   * Use Light DOM so theme CSS applies
   */
  protected override createRenderRoot() {
    return this
  }

  override connectedCallback() {
    super.connectedCallback()

    // Render icons (replace placeholders with UnoCSS icon spans)
    this.renderIcons()
  }

  /**
   * Render icon links by replacing placeholder spans with UnoCSS icon elements
   */
  private renderIcons() {
    const iconLinks = this.querySelectorAll('.footer-icon-link[data-icon]')

    iconLinks.forEach((link) => {
      const iconName = link.getAttribute('data-icon')
      const placeholder = link.querySelector('.icon-placeholder')

      if (!iconName || !placeholder) return

      const iconClass = this.getIconClass(iconName)
      if (iconClass) {
        const iconSpan = document.createElement('span')
        iconSpan.className = `footer-icon ${iconClass}`
        placeholder.replaceWith(iconSpan)
      }
    })
  }

  /**
   * Get UnoCSS icon class for a given icon name
   */
  private getIconClass(name: string): string {
    return ICON_CLASSES[name] ?? `i-lucide:${name}`
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'stencila-footer': StencilaFooter
  }
}
