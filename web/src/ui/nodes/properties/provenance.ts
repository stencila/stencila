import { css } from '@twind/core'
import { html, LitElement } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../../../twind'

/**
 * A component for displaying the `provenance` property of a node.
 */
@customElement('stencila-ui-node-provenance')
@withTwind()
export class UINodeProvenance extends LitElement {
  override render() {
    // apply flex to the slotted container
    const countStyles = css`
      & ::slotted([slot='provenance']) {
        display: flex;
        align-items: center;
        column-gap: 0.5rem;
      }
    `

    return html`
      <div class="mx-4 ${countStyles}">
        <slot></slot>
      </div>
    `
  }
}
