import { html, LitElement } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

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
    return html`
      <stencila-ui-inline-on-demand type="Object">
        <div slot="content">
          <slot></slot>
        </div>
      </stencila-ui-inline-on-demand>
    `
  }
}
