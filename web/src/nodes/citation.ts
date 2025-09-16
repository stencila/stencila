import { CitationMode, CompilationMessage } from '@stencila/types'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `Citation` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/prose/citation.md
 */
@customElement('stencila-citation')
@withTwind()
export class Citation extends Entity {
  @property({ attribute: 'target' })
  target?: string

  @property({ attribute: 'citation-mode' })
  citationMode?: CitationMode

  @property({ attribute: 'citation-prefix' })
  citationPrefix?: string

  @property({ attribute: 'citation-suffix' })
  citationSuffix?: string

  @property({ attribute: 'compilation-messages', type: Array })
  compilationMessages?: CompilationMessage[]

  override render() {
    const inner = this.compilationMessages ?
      html`<sl-tooltip placement="top" content="${this.compilationMessages.map((msg) => msg.message).join('; ')}"><span class="text-gray-700"><slot name="content"></slot></span></sl-tooltip>` :
      html`${this.citationPrefix ? `${this.citationPrefix} ` : ''}<slot name="cites"></slot>${this.citationSuffix ? ` ${this.citationSuffix}` : ''}`

    const items = this.closestGlobally('stencila-citation-group [slot=items]')
    if (items) {
      // Citation item within a citation group
      if (this != items.lastElementChild) {
        // Not last item in citation group, so add separator
        return html`${inner}; `
      } else {
        // Last item in citation group
        return html`${inner}`
      }
    } else if (this.citationMode == undefined || this.citationMode == 'Parenthetical') {
      // Parenthetical citation
      return html`(${inner})`
    } else {
      // Narrative citation (do not distinguish between Narrative and NarrativeAuthor but rather in Reference)
      return inner
    }
  }
}
