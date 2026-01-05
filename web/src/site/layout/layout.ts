import { LitElement } from 'lit'
import { customElement } from 'lit/decorators.js'

import './color-mode' // Color mode toggle component

/**
 * Site layout shell component
 *
 * Uses Light DOM so that theme CSS applies directly to layout elements.
 * The layout is responsive and hides sidebars at smaller breakpoints.
 */
@customElement('stencila-layout')
export class StencilaLayout extends LitElement {
  /**
   * Override to use Light DOM so theme CSS applies
   */
  protected override createRenderRoot() {
    return this
  }
}

declare global {
  interface HTMLElementTagNameMap {
    'stencila-layout': StencilaLayout
  }
}
