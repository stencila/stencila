import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `CiteGroup` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/cite-group.md
 */
@customElement('stencila-cite-group')
@withTwind()
export class CiteGroup extends Entity {
  override render() {
    return html`(<slot name="items"></slot>)`
  }
}
