import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'

import { Cite } from './cite'
import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `Reference` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/reference.md
 */
@customElement('stencila-reference')
@withTwind()
export class Reference extends Entity {
  @property()
  doi?: string

  @property({ attribute: '_title' })
  $title?: string

  @property()
  date?: string

  @property({ type: Array })
  authors?: string[]

  override render() {
    const cite = this.closestGlobally('stencila-cite') as Cite | null

    if (cite) {
      return this.renderWithinCite(cite)
    } else {
      return this.renderDefault()
    }
  }

  renderWithinCite(cite: Cite) {
    // TODO: Use author last name
    const author = this.authors[0]

    switch (cite.citationMode) {
      case 'Narrative':
        return html`${author} (${this.date})`
      case 'NarrativeAuthor':
        return html`${author}`
      case 'Parenthetical':
          return html`${author}, ${this.date}`
      default:
        return html`${author} ${this.date}`
    }
  }

  renderDefault() {
    return html`<div class="font-sans text-xs">
      ${this.authors ? this.authors.join(', ') : ''}${this.date
        ? html` (${this.date})`
        : ''}
      ${this.$title
        ? html`<span class="font-semibold"> ${this.$title}</span>`
        : ''}
      ${this.doi
        ? html` <a href="https://doi.org/${this.doi}" target="_blank"
            >${this.doi}</a
          >`
        : ''}
    </div>`
  }
}
