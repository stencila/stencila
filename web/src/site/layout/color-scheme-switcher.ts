import { LitElement, html } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import {
  ColorScheme,
  ColorSchemeManager,
} from '../../ui/document/color-scheme'

/**
 * Color scheme switcher component
 *
 * A Light DOM component that provides a toggle button to switch between
 * light and dark color schemes. The style can be icon-only, label-only,
 * or both, controlled by the data-style attribute.
 *
 * Styles are defined in themes/base/layout.css since this uses Light DOM.
 */
@customElement('stencila-color-scheme-switcher')
export class StencilaColorSchemeSwitcher extends LitElement {
  /**
   * Display style: 'icon', 'label', or 'both'
   */
  @property({ type: String, attribute: 'display-style' })
  displayStyle: 'icon' | 'label' | 'both' = 'icon'

  /**
   * Current color scheme
   */
  @state()
  private colorScheme: ColorScheme = 'light'

  /**
   * Use Light DOM so theme CSS applies
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
    // Show the TARGET state (what clicking will switch to), not current state
    const isCurrentlyDark = this.colorScheme === 'dark'
    const targetIcon = isCurrentlyDark ? 'i-lucide:sun' : 'i-lucide:moon'
    const targetLabel = isCurrentlyDark ? 'Light mode' : 'Dark mode'
    const ariaLabel = `Switch to ${isCurrentlyDark ? 'light' : 'dark'} mode`

    const showIcon = this.displayStyle !== 'label'
    const showLabel = this.displayStyle !== 'icon'

    return html`
      <button
        class="color-scheme-switcher"
        @click=${this.toggle}
        aria-label=${ariaLabel}
        title=${ariaLabel}
      >
        ${showIcon
          ? html`<span class="color-scheme-switcher-icon ${targetIcon}"></span>`
          : ''}
        ${showLabel
          ? html`<span class="color-scheme-switcher-label">${targetLabel}</span>`
          : ''}
      </button>
    `
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'stencila-color-scheme-switcher': StencilaColorSchemeSwitcher
  }
}
