import { html, LitElement } from 'lit'
import { customElement } from 'lit/decorators.js'

import '../ui/nodes/cards/inline-on-demand'

import { withTwind } from '../twind'

/**
 * Web component representing a Stencila Schema `UnsignedInteger` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/unsigned-integer.md
 */
@customElement('stencila-unsigned-integer')
@withTwind()
export class UnsignedInteger extends LitElement {
  override render() {
    return html`<stencila-ui-inline-on-demand type="UnsignedInteger"
      ><div slot="body"><slot></slot></div
    ></stencila-ui-inline-on-demand>`
  }
}
