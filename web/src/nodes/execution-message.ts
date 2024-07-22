import { MessageLevel } from '@stencila/types'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

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

  @property()
  message: string

  @property({ attribute: 'error-type' })
  errorType?: string

  @property({ attribute: 'stack-trace' })
  stackTrace?: string

  @property({ attribute: 'code-location', type: Array })
  codeLocation?: [number, number, number, number]

  override render() {
    return html`
      <stencila-ui-node-execution-message
        level=${this.level}
        message=${this.message}
        error-type=${this.errorType}
        stack-trace=${this.stackTrace}
      >
      </stencila-ui-node-execution-message>
    `
  }
}
