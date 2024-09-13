import { html, LitElement } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'
import { closestGlobally } from '../utilities/closestGlobally'

import '../ui/nodes/cards/inline-on-demand'

import './object-item'

/**
 * Web component representing a Stencila Schema `Object` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/object.md
 */
@customElement('stencila-object')
@withTwind()
export class Object extends LitElement {
  /**
   * render a node card with the value in the content slot.
   */
  override render() {
    return closestGlobally(this, 'stencila-code-expression')
      ? html`<slot></slot>`
      : html`<stencila-ui-inline-on-demand type="Object">
          <span slot="content"><slot></slot></span>
        </stencila-ui-inline-on-demand>`
  }
}
