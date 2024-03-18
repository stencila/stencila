import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import moment from 'moment'

import { withTwind } from '../../../../twind'

/**
 * UI Node Timestamp Property
 *
 * A component for displaying a timestamp (milliseconds since Unix epoch)
 * into a human readable form.
 */
@customElement('stencila-ui-node-timestamp-property')
@withTwind()
export class UINodeTimestampProperty extends LitElement {
  @property({ type: Number })
  value?: number | undefined

  /**
   * A string representation of the last execution,
   * relative to the current time
   */
  protected relativeTime: string = '-'

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
   * Get the timestamp as an ISO 8601 formatted string
   */
  protected isoFormat = (): string | null =>
    this.value ? moment(this.value).format('YYYY-MM-DD HH:mm:ss UTC') : null

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
    return html`${this.relativeTime}`
  }
}
