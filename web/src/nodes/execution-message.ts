import { MessageLevel } from '@stencila/types'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'
import '../ui/nodes/properties/generic/collapsible'

import { withTwind } from '../twind'

import '../ui/nodes/properties/execution-message'

import { Entity } from './entity'

/**
 * Web component representing a Stencila Schema `ExecutionMessage` node
 *
 * Note: This is a "pass-through" component: properties and slots are just passed through
 * to the `<stencila-ui-node-execution-message>` component. This is done to maintain
 * a consistent pattern in how we implement components for node properties.
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/execution-message.md
 */
@customElement('stencila-execution-message')
@withTwind()
export class ExecutionMessage extends Entity {
  @property()
  level: MessageLevel

  @property({ attribute: 'error-type' })
  errorType: string

  override render() {
    return html`
      <stencila-ui-node-execution-message
        level=${this.level}
        error-type=${this.errorType}
      >
        <slot name="message" slot="message"></slot>
        <slot name="stack-trace" slot="stack-trace"></slot>
      </stencila-ui-node-execution-message>
    `
  }
}
