import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'

import { Entity } from './entity'
import './helpers/node-authors'
import { nodeCardParentStyles, nodeCardStyles } from './helpers/node-card'

/**
 * Web component representing a Stencila Schema `Paragraph` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/paragraph.md
 */
@customElement('stencila-paragraph')
@withTwind()
export class Paragraph extends Entity {
  override renderStaticView() {
    const view = this.documentView()
    return html`
      <div class=${nodeCardParentStyles(view)}>
        <slot name="content"></slot>
        <stencila-node-card type="Paragraph" class=${nodeCardStyles(view)}>
          <stencila-node-authors type="Paragraph">
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
        <stencila-node-card type="Paragraph" class=${nodeCardStyles(view)}>
          <stencila-node-authors type="Paragraph">
            <slot name="authors"></slot>
          </stencila-node-authors>
        </stencila-node-card>
      </div>
    `
  }
}
