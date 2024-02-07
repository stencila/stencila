import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'

import { Entity } from './entity'
import './helpers/node-authors'
import { nodeCardParentStyles, nodeCardStyles } from './helpers/node-card'

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

    return html`<div class=${nodeCardParentStyles(view)}>
      ${view !== 'source' ? html`<slot name="content"></slot>` : ''}

      <stencila-node-card type="Heading" class=${nodeCardStyles(view)}>
        <stencila-node-authors type="Heading">
          <slot name="authors"></slot>
        </stencila-node-authors>
      </stencila-node-card>
    </div>`
  }
}
