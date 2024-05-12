import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import '../ui/nodes/card'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/authorship'
import '../ui/nodes/properties/provenance'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `Heading` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/heading.md
 */
@customElement('stencila-heading')
export class Heading extends Entity {
  @property({ type: Number })
  level: Number

  /**
   * In static view, just render the `content`.
   */
  override renderStaticView() {
    return html`<slot name="content"></slot>`
  }

  /**
   * In dynamic view, render the `content`, `authors` and summary stats in
   * a node card that is shown on demand.
   */
  override renderDynamicView() {
    // TODO: Add summary stats to card

    return html`
      <stencila-ui-block-on-demand type="Heading" view="dynamic">
        <div slot="body">
          <stencila-ui-node-authors type="Heading">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
          <stencila-ui-node-provenance type="Heading">
            <slot name="provenance"></slot>
          </stencila-ui-node-provenance>
        </div>
        <slot name="content" slot="content"></slot>
      </stencila-ui-block-on-demand>
    `
  }

  /**
   * In source view, render `authors` and summary stats in a node card. Do not render
   * `content` since that is visible in the source code.
   */
  override renderSourceView() {
    // TODO: Add summary stats to card

    return html`
      <stencila-ui-node-card type="Heading" view="source">
        <div slot="body">
          <stencila-ui-node-authors type="Heading">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </div>
      </stencila-ui-node-card>
    `
  }
}
