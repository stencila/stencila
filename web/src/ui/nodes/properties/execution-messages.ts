import { NodeType } from '@stencila/types'
import { LitElement, html } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { withTwind } from '../../../twind'

import './generic/collapsible'

/**
 * A component for displaying the `executionMessages` property of executable nodes
 */
@customElement('stencila-ui-node-execution-messages')
@withTwind()
export class UINodeExecutionMessages extends LitElement {
  @property()
  type: NodeType

  @state()
  private hasMessages: boolean = false

  private onSlotChange(event: Event): void {
    const slot = event.target as HTMLSlotElement
    this.hasMessages = slot.assignedElements({ flatten: true }).length > 0
  }

  override render() {
    return html`
      <stencila-ui-node-collapsible-property
        type=${this.type}
        icon-name="terminal"
        header-title="Messages"
        wrapper-css=${!this.hasMessages ? 'hidden' : ''}
        ?expanded=${this.hasMessages}
      >
        <div slot="content" class="flex flex-col gap-y-3">
          <slot @slotchange=${this.onSlotChange}></slot>
        </div>
      </stencila-ui-node-collapsible-property>
    `
  }
}
