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

  /**
   * A string representation of the last execution,
   * relative to the current time
   */
  private relativeTime: string = '-'

  /**
   * Interval used to update the relative time
   */
  private updateRelativeTimeInterval: NodeJS.Timeout

  /**
   * Set the `relativeTime` property and request the element update
   */
  private updateRelativeTime = () => {
    this.relativeTime =
      this.value === undefined || this.value === 0
        ? '-'
        : moment(this.value).fromNow()
    this.requestUpdate()
  }

  /**
   * When connected, set the relative time to update every minute
   */
  override connectedCallback(): void {
    super.connectedCallback()

    this.updateRelativeTime()

    this.updateRelativeTimeInterval = setInterval(() => {
      this.updateRelativeTime()
    }, 1000 * 60)
  }

  /**
   * Clear the interval
   */
  override disconnectedCallback(): void {
    super.disconnectedCallback()
    clearInterval(this.updateRelativeTimeInterval)
  }

  override render() {
    const isoFormat = this.value
      ? moment(this.value).format('YYYY-MM-DDTHH:mm:ss')
      : null

    return html`
      <stencila-ui-node-simple-property
        icon-name="clock"
        icon-library="default"
        tooltip-content="${isoFormat
          ? `Last execution ended at: ${isoFormat}`
          : 'No previous executions'}"
      >
        ${this.relativeTime}
      </stencila-ui-node-simple-property>
    `
  }
}
