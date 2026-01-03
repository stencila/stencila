import { LitElement } from 'lit'
import { customElement } from 'lit/decorators.js'

/**
 * Map of icon names to UnoCSS icon classes
 *
 * Uses simple-icons for brand logos and lucide for utility icons.
 * Icons are loaded at runtime via UnoCSS presetIcons from Iconify CDN.
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
 * Site header component
 *
 * A Light DOM component that enhances the server-rendered header with:
 * - Icon rendering (replaces placeholder spans with UnoCSS icon classes)
 *
 * The HTML is server-rendered for SEO and fast paint.
 * Note: Mobile navigation toggle is handled by stencila-layout, not the header.
 */
@customElement('stencila-header')
export class StencilaHeader extends LitElement {
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
   *
   * Icons are rendered as spans with UnoCSS icon classes that get processed
   * at runtime by UnoCSS presetIcons.
   */
  private renderIcons() {
    const iconLinks = this.querySelectorAll('.header-icon-link[data-icon]')

    iconLinks.forEach((link) => {
      const iconName = link.getAttribute('data-icon')
      const placeholder = link.querySelector('.icon-placeholder')

      if (!iconName || !placeholder) return

      const iconClass = this.getIconClass(iconName)
      if (iconClass) {
        const iconSpan = document.createElement('span')
        iconSpan.className = `header-icon ${iconClass}`
        placeholder.replaceWith(iconSpan)
      }
    })
  }

  /**
   * Get UnoCSS icon class for a given icon name
   *
   * Falls back to lucide icon set for unknown icons.
   */
  private getIconClass(name: string): string {
    return ICON_CLASSES[name] ?? `i-lucide:${name}`
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'stencila-header': StencilaHeader
  }
}
