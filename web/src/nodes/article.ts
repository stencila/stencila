import { LitElement, html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'

@customElement('stencila-article')
@withTwind()
export class StencilaArticle extends LitElement {
  override render() {
    return html`
      <slot name="provenance"></slot>
      <slot name="content"></slot>
    `
  }
}
