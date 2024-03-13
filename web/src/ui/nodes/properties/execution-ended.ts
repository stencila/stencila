import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'
import moment from 'moment'

import { withTwind } from '../../../twind'

import './generic/simple'

/**
 * A component for displaying the `executionEnded` property of executable nodes
 */
@customElement('stencila-ui-node-execution-ended')
@withTwind()
export class UINodeExecutionEnded extends LitElement {
  @property({ type: Number })
  value?: number | undefined

  override render() {
    // TODO: Add a setInterval or similar to refresh the humanized representation
    // every minute or so

    // TODO: show the ISO 8601 date/time in the tooltip

    const value =
      this.value === undefined || this.value === 0
        ? '-'
        : moment(this.value).fromNow()

    return html`
      <stencila-ui-node-simple-property
        icon-name="clock"
        icon-library="default"
        tooltip-content="Last execution ended at ${this.value}"
      >
        ${value}
      </stencila-ui-node-simple-property>
    `
  }
}
