import { MessageLevel } from '@stencila/types'
import { LitElement, html, css } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../twind'

@customElement('stencila-ui-node-execution-message')
@withTwind()
export class ExecutionMessage extends LitElement {
  @property()
  level: MessageLevel

  @property({ attribute: 'error-type' })
  errorType?: string

  static override styles = css`
    slot::slotted(pre) {
      padding: 1rem;
      margin-top: 0.5rem !important;
      background-color: white;
      border: 1px solid red;
      border-radius: 5px;
    }
    slot[name='message']::slotted(pre) {
      text-wrap: wrap;
    }
    slot[name='stack-trace']::slotted(pre) {
      overflow-x: auto;
    }
  `

  override render() {
    return html`
      <div>
        <slot name="message"></slot>
      </div>
      <div>
        <slot name="stack-trace"></slot>
      </div>
    `
  }
}
