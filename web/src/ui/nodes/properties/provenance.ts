import '@shoelace-style/shoelace/dist/components/icon/icon'
import { NodeType } from '@stencila/types'
import { html, LitElement } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { withTwind } from '../../../twind'

import './generic/collapsible-details'

/**
 * A component for displaying the `provenance` property of a node
 */
@customElement('stencila-ui-node-provenance')
@withTwind()
export class UINodeProvenance extends LitElement {
  /**
   * The type of node that the `authors` property is on
   *
   * Used to determine the styling of this component.
   */
  @property()
  type: NodeType

  /**
   * Whether there are any authors in the list
   *
   * Used to determine if anything should be rendered.
   */
  @state()
  private hasItems = false

  override firstUpdated(changedProperties: Map<string, string | boolean>) {
    super.firstUpdated(changedProperties)

    const slot: HTMLSlotElement = this.shadowRoot.querySelector('slot')
    if (slot) {
      this.hasItems = slot.assignedElements({ flatten: true }).length !== 0
    }
  }

  override render() {
    return html`<stencila-ui-node-collapsible-details
      type=${this.type}
      icon-name="handshake"
      icon-library="lucide"
      title="Provenance"
    >
      <slot></slot>
    </stencila-ui-node-collapsible-details>`
  }
}
