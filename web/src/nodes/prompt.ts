import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'

@customElement('stencila-prompt')
@withTwind()
export class StencilaPrompt extends LitElement {
  /**
   * Indicates that this is the root node of the document
   */
  @property({ type: Boolean })
  root: boolean

  override render() {
    return html`
      <slot name="content"></slot>
    `
  }
}
