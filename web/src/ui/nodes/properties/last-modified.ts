import '@shoelace-style/shoelace/dist/components/tooltip/tooltip'
import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../../../twind'

import { UINodeTimestampProperty } from './generic/timestamp'

/**
 * A component for displaying the `lastModified` property of an `AuthorRole` node
 * 
 * If the timestamp value is undefined/null then renders an empty string
 */
@customElement('stencila-ui-node-last-modified')
@withTwind()
export class UINodeLastModified extends UINodeTimestampProperty {
  override render() {
    if (!this.value) {
      return html``
    }

    const isoFormat = this.isoFormat()
    return html`
      <sl-tooltip
        content=${`Last modified at ${isoFormat}`}
        .disabled=${isoFormat === null}
      >
        <span>${this.relativeTime}</span>
      </sl-tooltip>
    `
  }
}
