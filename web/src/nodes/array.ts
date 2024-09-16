import { html, LitElement } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'
import { closestGlobally } from '../utilities/closestGlobally'

import '../ui/nodes/cards/inline-on-demand'

import './array-item'

/**
 * Web component representing a Stencila Schema `Array` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/array.md
 */
@customElement('stencila-array')
@withTwind()
export class Array extends LitElement {
  override render() {
    return closestGlobally(this, 'stencila-code-expression')
      ? html`<slot></slot>`
      : html`<stencila-ui-inline-on-demand type="Array">
          <span slot="content"><slot></slot></span>
        </stencila-ui-inline-on-demand>`
  }
}
