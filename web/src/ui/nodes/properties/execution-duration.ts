import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'
import moment from 'moment'

import { withTwind } from '../../../twind'

import './generic/simple'

/**
 * A component for displaying the `executionDuration` property of executable nodes
 */
@customElement('stencila-ui-node-execution-duration')
@withTwind()
export class UINodeExecutionDuration extends LitElement {
  @property({ type: Number })
  value?: number = undefined

  override render() {
    const value =
      this.value === undefined || this.value === 0
        ? '-'
        : moment.duration(this.value, 'ms').humanize()

    return html`
      <stencila-ui-node-simple-property
        icon-name="stopwatch"
        icon-library="default"
        tooltip-content="Duration of last execution"
      >
        ${value}
      </stencila-ui-node-simple-property>
    `
  }
}
