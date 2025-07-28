import { CitationMode, CompilationMessage } from '@stencila/types'
import { html } from 'lit'
import { customElement, property, state } from 'lit/decorators'

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

  @property({ attribute: 'compilation-messages', type: Array })
  compilationMessages?: CompilationMessage[]

  /**
   * Whether the citation has a resolved `Reference` in the `cites` slot
   *
   * `Citation` nodes that originate from sources such as JATS can has both a resolved `cites` property
   * (based on `target`) and `content`. The `content` is treated as a fallback and will not be shown
   * if the cite has a resolved `reference`.
   */
  @state()
  hasCites: boolean = false

  onCitesSlotChange({ target: slot }: Event) {
    const citesElem = (slot as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0]
    this.hasCites = !!citesElem
  }

  override render() {
    const inner = this.compilationMessages ? 
      html`<sl-tooltip placement="top" content="${this.compilationMessages.map(msg => msg.message).join('; ')}"><span class="text-gray-500">${this.target}</span></sl-tooltip>`: 
      html`<slot
        name="cites"
        @slotchange=${this.onCitesSlotChange}
      ></slot
      ><span class=${this.hasCites ? 'hidden' : ''}
        ><slot name="content"></slot
      ></span>`

    if (this.citationMode == 'Parenthetical') {
      return html`(${inner})`
    }

    const items = this.closestGlobally('stencila-citation-group [slot=items]')
    if (items) {
      if (this != items.lastElementChild) {
        return html`${inner}; `
      }
    }

    return html`${inner}`
  }
}
