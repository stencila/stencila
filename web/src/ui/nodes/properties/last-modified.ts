import '@shoelace-style/shoelace/dist/components/tooltip/tooltip'
import { css, html } from 'lit'
import { customElement, property } from 'lit/decorators'

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
  @property({ attribute: 'role-label' })
  roleLabel: string = 'Last modified'

  static override styles = css`
    :host {
      display: var(--ui-lastmod-display);
      position: relative;
    }
  `

  override render() {
    if (!this.value) {
      return html``
    }

    const isoFormat = this.isoFormat()

    return html`
      <sl-tooltip
        content=${`${this.roleLabel} at ${isoFormat}`}
        .disabled=${isoFormat === null}
      >
        <span class="select-none">${this.relativeTime}</span>
      </sl-tooltip>
    `
  }
}
