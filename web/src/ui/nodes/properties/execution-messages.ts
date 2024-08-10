import { MessageLevel, NodeType } from '@stencila/types'
import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { withTwind } from '../../../twind'
import { executionMessageUI, nodeUi } from '../icons-and-colours'

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
        icon-library="default"
        header-title="Messages"
        wrapper-css=${!this.hasMessages ? 'hidden' : ''}
        ?collapsed=${this.hasMessages}
      >
        <div slot="content" class="flex flex-col gap-y-3">
          <slot @slotchange=${this.onSlotChange}></slot>
        </div>
      </stencila-ui-node-collapsible-property>
    `
  }

  renderLozenge = (level: MessageLevel, count: number) => {
    const { icon, colour } = executionMessageUI(level)
    const { colour: nodeColour } = nodeUi(this.type)

    const styles = apply([
      'flex items-center',
      `bg-[${nodeColour}]`,
      `text-xs text-${colour}`,
      'px-1.5 ml-1',
      'rounded-full',
    ])

    return html`
      <div class=${styles}>
        <sl-ui-icon name=${icon}></sl-ui-icon>
        <span class="ml-1">${count}</span>
      </div>
    `
  }
}
