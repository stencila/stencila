import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'
import { nodeCardParentStyles, nodeCardStyles } from '../ui/nodes/card'
import '../ui/nodes/properties/authors'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `List` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/list.md
 */
@customElement('stencila-list')
@withTwind()
export class List extends Entity {
  override renderStaticView() {
    const view = this.documentView()
    return html`
      <div class=${nodeCardParentStyles(view)}>
        <slot name="items"></slot>
        <stencila-ui-node-card type="List" view="static">
          <stencila-ui-node-authors type="List">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </stencila-ui-node-card>
      </div>
    `
  }

  override renderDynamicView() {
    return this.renderStaticView()
  }

  override renderVisualView() {
    return this.renderDynamicView()
  }

  override renderSourceView() {
    const view = this.documentView()
    return html`
      <div class=${nodeCardParentStyles(view)}>
        <stencila-ui-node-card type="List" view="source">
          <stencila-ui-node-authors type="List">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </stencila-ui-node-card>
      </div>
    `
  }
}
