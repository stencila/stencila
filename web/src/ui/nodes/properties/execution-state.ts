import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../twind'

import './generic/simple'

/**
 * A component for displaying both the `executionStatus` and `executionRequired`
 * properties of executable nodes
 *
 * These two properties are combined into a single component because it is
 * unlikely that we'll need to display both of them.
 */
@customElement('stencila-ui-node-execution-state')
@withTwind()
export class UINodeExecutionState extends LitElement {
  @property()
  status: string

  @property()
  required: string

  @property({ type: Number })
  count: number

  private getIconName() {
    switch (this.status) {
      case 'Running':
        return 'arrowClockwise'
      case 'Warnings':
      case 'Errors':
      case 'Exceptions':
        return 'exclamationCircle'
      case 'Succeeded':
        return 'checkCircle'
      default:
        return 'activity'
    }
  }

  override render() {
    const [status, tooltip] = !this.status
      ? this.required === 'NeverExecuted' || !this.count
        ? ['Not Executed', 'Node has not been executed yet']
        : [this.status, 'Status of the last execution']
      : [this.status, 'Status of the last execution']

    const icon = this.getIconName()

    // TODO: Decide how best to coalesce (or not) `status` and `required` including
    // labels and iconography. Currently, only status is being shown.
    return html`
      <stencila-ui-node-simple-property
        icon=${icon}
        icon-classes=${icon === 'arrowClockwise' ? 'animate-spin' : ''}
        tooltip=${tooltip}
      >
        ${status}
      </stencila-ui-node-simple-property>
    `
  }
}
