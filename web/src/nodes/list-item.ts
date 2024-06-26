import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `ListItem` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/list-item.md
 */
@customElement('stencila-list-item')
export class ListItem extends Entity {
  override render() {
    return html`<slot name="content"></slot>`
  }
}
