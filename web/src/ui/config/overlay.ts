import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../../twind'

/**
 * UI Overlay
 *
 * An overlay to place on top of the rest of the UI while a dialog is open
 */
@customElement('stencila-ui-overlay')
@withTwind()
export class UIOverlay extends LitElement {
  @property({ type: Boolean })
  isOpen: boolean = false

  @property()
  handleClose: () => void | undefined

  override render() {
    return html`<div
      class="transition w-screen h-screen overflow-none fixed top-0 left-0  z-10 bg-white ${this
        .isOpen
        ? 'opacity-50 pointer-events-all cursor-pointer'
        : 'opacity-0 pointer-events-none'}"
      @click=${this.handleClose}
    ></div>`
  }
}
