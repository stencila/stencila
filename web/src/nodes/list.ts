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
  render() {
    return html`
      <slot name="items"></slot>

      <stencila-block-infobox icon="list" title="List">
        <slot name="authors" slot="authors"></slot>
      </stencila-block-infobox>
    `
  }
}
