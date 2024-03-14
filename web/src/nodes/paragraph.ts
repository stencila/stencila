import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'
import { nodeCardParentStyles, nodeCardStyles } from '../ui/nodes/card'
import '../ui/nodes/properties/authors'

import { Entity } from './entity'

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
      </div>
    `
  }
  // <stencila-ui-node-card type="Paragraph" class=${nodeCardStyles(view)}>
  //   <stencila-ui-node-authors type="Paragraph">
  //     <slot name="authors"></slot>
  //   </stencila-ui-node-authors>
  // </stencila-ui-node-card>

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
        <stencila-ui-node-card type="Paragraph" class=${nodeCardStyles(view)}>
          <slot name="authors"></slot>
        </stencila-ui-node-card>
      </div>
    `
  }
}
