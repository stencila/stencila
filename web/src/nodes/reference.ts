import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'

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
    return html`<div class="font-sans text-xs">
      ${this.authors ? this.authors.join(', ') : ''}${this.date
        ? html` (${this.date})`
        : ''}
      ${this.$title
        ? html`<span class="font-semibold"> ${this.$title}</span>`
        : ''}
      ${this.doi
        ? html` <a href=${this.doi} target="_blank">${this.doi}</a>`
        : ''}
    </div>`
  }
}
