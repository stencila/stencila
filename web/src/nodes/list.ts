import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'

import { Entity } from './entity'
import './helpers/node-authors'
import { nodeCardParentStyles, nodeCardStyles } from './helpers/node-card'

/**
 * Web component representing a Stencila Schema `List` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/list.md
 */
@customElement('stencila-list')
@withTwind()
export class List extends Entity {
  override render() {
    const view = this.documentView()

    return html`<div class=${nodeCardParentStyles(view)}></div>
      ${view !== 'source' ? html`<slot name="items"></slot>` : ''}

      <stencila-node-card type="List" class=${nodeCardStyles(view)}>
        <stencila-node-authors type="List">
          <slot name="authors"></slot>
        </stencila-node-authors>
      </stencila-node-card>
    </div>`
  }
}
