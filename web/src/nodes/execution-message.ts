import '@shoelace-style/shoelace/dist/components/icon/icon'
import { MessageLevel } from '@stencila/types'
import { html, css } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `ExecutionMessage` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/execution-message.md
 */
@customElement('stencila-execution-message')
@withTwind()
export class ExecutionMessage extends Entity {
  @property({ type: String })
  level: MessageLevel

  // TODO: factor this out into <stencila-ui-node-execution-messages> (note plural because
  // that's the name of the property.

  // override the styles property to set the `<pre>` styles
  // TODO - sort this out, get the pre elements wrapped in divs from the server
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
      <stencila-ui-node-collapsible-property
        icon-name="terminal"
        icon-library="default"
        class="my-1"
      >
        <span slot="title">Messages</span>
        <div slot="content" class="overflow-hidden py-2">
          <div>
            <slot name="message"></slot>
          </div>
          <div>
            <slot name="stack-trace"></slot>
          </div>
        </div>
      </stencila-ui-node-collapsible-property>
    `
  }
}
