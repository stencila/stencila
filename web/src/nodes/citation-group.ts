import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `CitationGroup` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/citation-group.md
 */
@customElement('stencila-citation-group')
@withTwind()
export class CitationGroup extends Entity {
  override render() {
    return html`<slot name="content"></slot>`
  }
}
