import { CitationMode } from '@stencila/types'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `Cite` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/cite.md
 */
@customElement('stencila-cite')
@withTwind()
export class Cite extends Entity {
  @property({ attribute: 'citation-mode' })
  citationMode?: CitationMode

  override render() {
    return this.citationMode == 'Parenthetical'
      ? html`(<slot name="reference"></slot>)`
      : html`<slot name="reference"></slot>`
  }
}
