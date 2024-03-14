import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'

import '../ui/nodes/card'
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
  /**
   * In static view just render the `content`.
   */
  override renderStaticView() {
    return html`<slot name="content"></slot>`
  }

  /**
   * In dynamic view render `content`, and `authors` and summary stats in a node card
   * that is shown on hover.
   */
  override renderDynamicView() {
    // TODO: Add summary stats to card

    return html`
      <stencila-ui-node-card type="Paragraph" view="source">
        <div slot="body">
          <stencila-ui-node-authors type="Paragraph">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </div>
      </stencila-ui-node-card>

      <slot name="content"></slot>
    `
  }

  /**
   * In source view render `authors` and summary stats in a node card. Do not render
   * `content` since that is visible in the source code.
   */
  override renderSourceView() {
    // TODO: Add summary stats to card

    return html`
      <stencila-ui-node-card type="Paragraph" view="source">
        <div slot="body">
          <stencila-ui-node-authors type="Paragraph">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
        </div>
      </stencila-ui-node-card>
    `
  }
}
