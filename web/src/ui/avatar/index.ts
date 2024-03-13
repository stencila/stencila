import { LitElement, html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../../twind'

/**
 * UI Avatar
 *
 * An element that displays a picture of a person / AI agent.
 */
@customElement('stencila-ui-avatar')
@withTwind()
export class UIAvatar extends LitElement {
  override render() {
    return html`<picture>
      <img src="https://placehold.co/24" />
    </picture>`
  }
}
