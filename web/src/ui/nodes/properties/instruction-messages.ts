import { NodeType } from '@stencila/types'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../twind'

import './generic/collapsible'

/**
 * A component for displaying the `messages` property of `Instruction` nodes
 */
@customElement('stencila-ui-node-instruction-messages')
@withTwind()
export class UINodeInstructionMessages extends LitElement {
  @property()
  type: NodeType

  override render() {
    return html`
      <stencila-ui-node-collapsible-property
        type=${this.type}
        icon-name="chat-left-dots"
        icon-library="default"
      >
        <div slot="title">
          <span>Chat</span>
        </div>
        <div slot="content" class="p-3">
          <slot></slot>
        </div>
      </stencila-ui-node-collapsible-property>
    `
  }
}
