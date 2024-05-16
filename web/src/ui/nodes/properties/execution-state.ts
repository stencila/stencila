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

  override render() {
    // TODO: Decided how best to coalesce (or not) `status` and `required` including
    // labels and iconography. Currently, only status is being shown.
    return html`
      <stencila-ui-node-simple-property
        icon-name="lightning"
        icon-library="default"
        tooltip-content="The status of the last execution"
      >
        <span>${this.status}</span>
      </stencila-ui-node-simple-property>
    `
  }
}
