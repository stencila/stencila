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
  override renderStaticView() {
    const view = this.documentView()
    return html`
      <div class=${nodeCardParentStyles(view)}>
        <slot name="items"></slot>
        <stencila-node-card type="List" class=${nodeCardStyles(view)}>
          <stencila-node-authors type="List">
            <slot name="authors"></slot>
          </stencila-node-authors>
        </stencila-node-card>
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
        <stencila-node-card type="List" class=${nodeCardStyles(view)}>
          <stencila-node-authors type="List">
            <slot name="authors"></slot>
          </stencila-node-authors>
        </stencila-node-card>
      </div>
    `
  }
}
