import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import moment from 'moment'

import { withTwind } from '../../../../twind'

/**
 * UI Node Timestamp Property
 *
 * A lit element that converts a timstamp into a human readable form.
 */
@customElement('stencila-ui-node-timestamp-property')
@withTwind()
export class UINodeTimestamp extends LitElement {
  @property({ type: Number })
  timestamp: number

  override render() {
    return html`${moment(this.timestamp).fromNow()}`
  }
}
