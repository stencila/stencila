import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'
import { nodeCardStyles } from '../ui/nodes/card'
import '../ui/nodes/properties/authors'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `Heading` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/heading.md
 */
@customElement('stencila-heading')
@withTwind()
export class Heading extends Entity {
  @property({ type: Number })
  level: Number

  override render() {
    const view = this.documentView()

    return html`<div>
      ${view !== 'source' ? html`<slot name="content"></slot>` : ''}

      <stencila-ui-node-card type="Heading" class=${nodeCardStyles(view)}>
        <stencila-ui-node-authors type="Heading">
          <slot name="authors"></slot>
        </stencila-ui-node-authors>
      </stencila-ui-node-card>
    </div>`
  }
}
