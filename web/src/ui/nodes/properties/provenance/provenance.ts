import '@shoelace-style/shoelace/dist/components/icon/icon'
import { css } from '@twind/core'
import { html, LitElement, PropertyValues } from 'lit'
import { customElement, state } from 'lit/decorators'

import { withTwind } from '../../../../twind'

import '../../node-card/section-header'
import '../generic/collapsible-details'

/**
 * A component for displaying the `provenance` property of a node.
 */
@customElement('stencila-ui-node-provenance')
@withTwind()
export class UINodeProvenance extends LitElement {
  /**
   * Whether there are any provenance in the list
   *
   * Used to determine if anything should be rendered.
   */
  @state()
  private hasItems = false

  protected override firstUpdated(changedProperties: PropertyValues): void {
    super.firstUpdated(changedProperties)

    const slot = this.shadowRoot.querySelector('slot')
    if (slot) {
      this.hasItems = slot.assignedElements().length !== 0
    }
  }

  override render() {
    // apply flex to the slotted container
    const countStyles = css`
      & ::slotted([slot='provenance']) {
        display: flex;
        column-gap: 0.5rem;
      }
    `

    return html`
      <div class="mx-4 ${countStyles} ${!this.hasItems ? 'hidden' : ''}">
        <slot></slot>
      </div>
    `
  }
}
