import { LitElement, html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

/**
 * UI Insert block
 *
 * A container to hold inserted blocks
 */
@customElement('stencila-insert-block')
@withTwind()
export class UIInsertBlock extends LitElement {
  protected override render() {
    return html`
      <div class="w-full">
        <slot name="content"></slot>
      </div>
    `
  }
}
