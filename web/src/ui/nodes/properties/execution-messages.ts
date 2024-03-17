import { MessageLevel, NodeType } from '@stencila/types'
import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../twind'
import './generic/collapsible'
import { executionMessageUI, nodeUi } from '../icons-and-colours'

/**
 * A component for displaying the `executionMessages` property of executable nodes
 */
@customElement('stencila-ui-node-execution-messages')
@withTwind()
export class UINodeExecutionMessages extends LitElement {
  // TODO: implement summary icons of the number of messages of each level

  @property({ type: String })
  type: NodeType

  // properties for the message type count
  @property({ type: Number, attribute: 'warn-count' })
  warnCount: number = 0

  @property({ type: Number, attribute: 'error-count' })
  errorCount: number = 0

  @property({ type: Number, attribute: 'debug-count' })
  debugCount: number = 0

  override render() {
    const { borderColour } = nodeUi(this.type)

    return html`
      <stencila-ui-node-collapsible-property
        .collapsed=${false}
        icon-name="terminal"
        icon-library="default"
        header-bg=${borderColour}
      >
        ${this.renderHeader()}
        <div class="flex flex-col gap-y-3" slot="content">
          <slot class="messages-slot"></slot>
        </div>
      </stencila-ui-node-collapsible-property>
    `
  }

  renderHeader = () => {
    const styles = apply(['flex justify-between', 'mr-1'])

    return html`
      <div class=${styles} slot="title">
        <span>Messages</span>
        <div class="flex">
          ${this.renderLozenge('Error', this.errorCount)}
          ${this.renderLozenge('Warn', this.warnCount)}
          ${this.renderLozenge('Debug', this.debugCount)}
        </div>
      </div>
    `
  }

  renderLozenge = (level: MessageLevel, count: number) => {
    const { icon, colour } = executionMessageUI(level)
    const { colour: nodeColour } = nodeUi(this.type)

    const styles = apply([
      'flex items-center',
      `bg-[${nodeColour}]`,
      `text-sm text-${colour}`,
      'px-1.5 ml-1',
      'rounded-full',
    ])

    return html`
      <div class=${styles}>
        <sl-icon name=${icon} library="default"></sl-icon>
        <span class="ml-1">${count}</span>
      </div>
    `
  }
}
