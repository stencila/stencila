import { NodeType } from '@stencila/types'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../twind'
import './generic/collapsible'
import { nodeUi } from '../icons-and-colours'

/**
 * A component for displaying the `executionMessages` property of executable nodes
 */
@customElement('stencila-ui-node-execution-messages')
@withTwind()
export class UINodeExecutionMessages extends LitElement {
  // TODO: implement summary icons of the number of messages of each level

  @property({ type: String })
  type: NodeType

  override render() {
    const { borderColour } = nodeUi(this.type)

    return html`
      <stencila-ui-node-collapsible-property
        .collapsed=${false}
        icon-name="terminal"
        icon-library="default"
        header-bg=${borderColour}
      >
        <span slot="title">Messages</span>
        <div class="flex flex-col gap-y-3" slot="content">
          <slot></slot>
        </div>
      </stencila-ui-node-collapsible-property>
    `
  }
}
