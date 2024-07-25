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

  override render() {
    const [statusString, tooltipContent] =
      this.required === 'NeverExecuted' || !this.count
        ? ['Not Executed', 'Node has not been executed yet']
        : [this.status, 'Status of the last execution']

    // TODO: Decide how best to coalesce (or not) `status` and `required` including
    // labels and iconography. Currently, only status is being shown.
    return html`
      <stencila-ui-node-simple-property
        icon-name="lightning"
        icon-library="default"
        tooltip-content=${tooltipContent}
      >
        <span>${statusString}</span>
      </stencila-ui-node-simple-property>
    `
  }
}
