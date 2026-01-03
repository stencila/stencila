import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'
import '../ui/document/article-references'

import { Entity } from './entity'

@customElement('stencila-article')
@withTwind()
export class Article extends Entity {
    @property()
  doi?: string

  @property({ type: Array })
  identifiers?: unknown[]

  @property({ attribute: 'date-published' })
  datePublished?: string

  @property({ attribute: 'date-accepted' })
  dateAccepted?: string

  @property({ attribute: 'date-created' })
  dateCreated?: string

  @property()
  repository?: string

  @property()
  path?: string

  @property()
  commit?: string

  override render() {
    return this.isRoot() ? this.renderAsRoot() : this.renderCard()
  }

  renderAsRoot() {
    return html`
      <slot name="title"></slot>
      <slot name="authors"></slot>
      <slot name="abstract"></slot>
      <slot name="content"></slot>

      <stencila-ui-article-references>
        <slot name="references"></slot>
      </stencila-ui-article-references>
    `
  }

  override renderCard() {
    return html`
      <stencila-ui-block-on-demand
        type="Article"
        node-id=${this.id}
        depth=${this.depth}
        ?has-root=${this.hasRoot()}
      >
        <div slot="content">
          <slot name="title"></slot>
          <slot name="abstract"></slot>
          <slot name="content"></slot>
        </div>
      </stencila-ui-block-on-demand>
    `
  }
}
