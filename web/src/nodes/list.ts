import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { Entity } from './entity'

import './helpers/block-infobox'

/**
 * Web component representing a Stencila Schema `List` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/list.md
 */
@customElement('stencila-list')
export abstract class List extends Entity {
  override render() {
    return html`
      ${this.documentView() !== 'source'
        ? html`<slot name="items"></slot>`
        : ''}

      <stencila-block-infobox title="List" currentNode="List">
        <slot name="authors" slot="authors"></slot>
        ${this.documentView() === 'source'
          ? html`<slot name="items" slot="items"></slot>`
          : ''}
      </stencila-block-infobox>
    `
  }
}
