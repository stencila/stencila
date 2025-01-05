import { LitElement, html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'

@customElement('stencila-prompt')
@withTwind()
export class StencilaPrompt extends LitElement {
  override render() {
    return html` <slot name="content"></slot> `
  }
}
