import { LitElement, html } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'
import moment from 'moment'

import { withTwind } from '../../../../twind'
import { getModeParam } from '../../../../utilities/getModeParam'

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
   * A string representation of the timestamp relative to the current time
   *
   * This is a `@state` so that any changes to it will trigger a re-render.
   */
  @state()
  protected relativeTime: string = '-'

  /**
   * Interval used to update the relative time
   */
  private updateInterval: NodeJS.Timeout

  /**
   * Set the `relativeTime` property and request the element update
   */
  private updateRelativeTime = () => {
    const windowMode = getModeParam(window)

    // if in test mode, set an arbitrary value for consistency across the screenshots
    if (windowMode && windowMode === 'test-expand-all') {
      this.relativeTime = 'some time ago'
      return
    }

    this.relativeTime =
      this.value === undefined || this.value === 0
        ? '-'
        : moment(this.value).fromNow()
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

    this.updateInterval = setInterval(() => {
      this.updateRelativeTime()
    }, 1000 * 60)
  }

  /**
   * When disconnected, clear the interval
   */
  override disconnectedCallback(): void {
    super.disconnectedCallback()

    clearInterval(this.updateInterval)
  }

  /**
   * Update the relative time when `value` changes
   */
  override update(changedProperties: Map<string, unknown>) {
    super.update(changedProperties)

    if (changedProperties.has('value')) {
      this.updateRelativeTime()
    }
  }

  override render() {
    return html`${this.relativeTime}`
  }
}
