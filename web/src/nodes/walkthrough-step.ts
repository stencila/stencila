import { html, LitElement } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../twind'

import '../ui/nodes/cards/inline-on-demand'

/**
 * Web component representing a Stencila Schema `WalkthroughStep` node
 *
 * This component currently only exists to turn on/off visibility of the
 * content of a walkthrough step (based on `isActive`).
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/data/walkthrough-step.md
 */
@customElement('stencila-walkthrough-step')
@withTwind()
export class WalkthroughStep extends LitElement {
  @property({ attribute: 'is-active' })
  isActive?: string

  override render() {
    return html`<div class=${this.isActive == 'true' ? '' : 'hidden'}>
      <slot name="content"></slot>
    </div>`
  }
}
