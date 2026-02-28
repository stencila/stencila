import { LitElement, html } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import {
  ColorScheme,
  ColorSchemeManager,
} from '../../ui/document/color-scheme'

/**
 * Color scheme switcher component
 *
 * A component that provides a toggle button to switch between light and dark
 * color schemes. The style can be icon-only, label-only, or both, controlled by
 * the style attribute.
 */
@customElement('stencila-color-mode')
export class StencilaColorMode extends LitElement {
  /**
   * Display style: 'icon', 'label', or 'both'
   */
  @property({ type: String, attribute: 'style' })
  displayStyle: 'icon' | 'label' | 'both' = 'icon'

  /**
   * Current color scheme
   */
  @state()
  private colorScheme: ColorScheme = 'light'

  /**
   * Use Light DOM so UnoCSS icons can be used
   */
  protected override createRenderRoot() {
    return this
  }

  override connectedCallback() {
    super.connectedCallback()

    // Load initial color scheme
    this.colorScheme = ColorSchemeManager.loadColorSchemePreference()

    // Listen for external color scheme changes
    window.addEventListener(
      'stencila-color-scheme-changed',
      this.handleColorSchemeChanged
    )
  }

  override disconnectedCallback() {
    super.disconnectedCallback()
    window.removeEventListener(
      'stencila-color-scheme-changed',
      this.handleColorSchemeChanged
    )
  }

  /**
   * Handle external color scheme changes
   */
  private handleColorSchemeChanged = () => {
    this.colorScheme = ColorSchemeManager.loadColorSchemePreference()
  }

  /**
   * Toggle between light and dark color schemes
   */
  private toggle() {
    const newScheme: ColorScheme =
      this.colorScheme === 'light' ? 'dark' : 'light'

    this.colorScheme = newScheme
    // Persist first so event listeners read the correct value
    ColorSchemeManager.persistColorScheme(newScheme)
    ColorSchemeManager.applyColorScheme(newScheme)
  }

  protected override render() {
    const isCurrentlyDark = this.colorScheme === 'dark'
    const targetIcon = isCurrentlyDark ? 'i-lucide:sun' : 'i-lucide:moon'
    const targetLabel = isCurrentlyDark ? 'Light' : 'Dark'
    const ariaLabel = `Switch to ${isCurrentlyDark ? 'light' : 'dark'} mode`

    const showIcon = this.displayStyle !== 'label'
    const showLabel = this.displayStyle !== 'icon'

    return html`
      <div
        class="toggle"
        role="button"
        tabindex="0"
        @click=${this.toggle}
        @keydown=${(e: KeyboardEvent) => {
          if (e.key === 'Enter' || e.key === ' ') {
            e.preventDefault()
            this.toggle()
          }
        }}
        aria-label=${ariaLabel}
        title=${ariaLabel}
      >
        ${showIcon ? html`<span class="${targetIcon} icon"></span>` : ''}
        ${showLabel ? html`<span class="label">${targetLabel}</span>` : ''}
      </div>
    `
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'stencila-color-mode': StencilaColorMode
  }
}
