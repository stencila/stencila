import '@shoelace-style/shoelace/dist/components/icon/icon'
import { NodeType } from '@stencila/types'
import { css } from '@twind/core'
import { html, LitElement, PropertyValues } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { withTwind } from '../../../../twind'
import { nodeUi } from '../../icons-and-colours'

import '../../node-card/section-header'
import '../generic/collapsible-details'

/**
 * A component for displaying the `provenance` property of a node.
 */
@customElement('stencila-ui-node-provenance')
@withTwind()
export class UINodeProvenance extends LitElement {
  /**
   * The type of node that the `provenance` property is on
   *
   * Used to determine the styling of this component.
   */
  @property()
  type: NodeType

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
    const { borderColour: headerBg } = nodeUi(this.type)

    // apply flex to the slotted container
    const countStyles = css`
      & ::slotted([slot='provenance']) {
        display: flex;
        column-gap: 0.5rem;
      }
    `

    return html`<div class="border-b border-black/20">
      <stencila-ui-node-card-section-header
        icon-name="handshake"
        icon-library="lucide"
        headerBg=${headerBg}
        wrapper-css="flex-wrap gap-y-2 ${this.hasItems ? '' : 'hidden'}"
      >
        <div slot="title" class="not-italic">Provenance</div>
        <div slot="right-side" class="${countStyles}">
          <slot></slot>
        </div>
      </stencila-ui-node-card-section-header>
    </div>`
  }
}
