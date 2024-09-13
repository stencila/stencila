import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../../../twind'

import { UINodeTimestampProperty } from './generic/timestamp'
import './generic/simple'

/**
 * A component for displaying the `executionEnded` property of executable nodes
 */
@customElement('stencila-ui-node-execution-ended')
@withTwind()
export class UINodeExecutionEnded extends UINodeTimestampProperty {
  override render() {
    const isoFormat = this.isoFormat()

    return html`
      <stencila-ui-node-simple-property
        icon-name="clock"
        tooltip-content="${isoFormat
          ? `Last executed at ${isoFormat}`
          : 'No previous executions'}"
      >
        ${this.relativeTime}
      </stencila-ui-node-simple-property>
    `
  }
}
