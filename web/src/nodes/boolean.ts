import { html, LitElement } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import '../ui/nodes/cards/inline-on-demand'

/**
 * Web component representing a Stencila Schema `Boolean` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/boolean.md
 */
@customElement('stencila-boolean')
@withTwind()
export class Boolean extends LitElement {
  override render() {
    return html`
      <stencila-ui-inline-on-demand type="Boolean">
        <div slot="content" class="w-full"><slot></slot></div>
      </stencila-ui-inline-on-demand>
    `
  }
}
