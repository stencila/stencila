import { LitElement, html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../../twind'

/**
 * UI sidebar
 *
 * The sidebar displayed in the main UI
 */
@customElement('stencila-ui-sidebar')
@withTwind()
export class UISidebar extends LitElement {
  override render() {
    return html`<div
      class="w-16 flex flex-col items-center justify-between mt-20 h-full max-h-[calc(100vh-5rem)] pb-5"
    >
      <stencila-ui-icon-button icon="sidebar"></stencila-ui-icon-button>
      <stencila-ui-icon-button icon="settings"></stencila-ui-icon-button>
    </div> `
  }
}