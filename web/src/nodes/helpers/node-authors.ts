import '@shoelace-style/shoelace/dist/components/icon/icon'
import { NodeType } from '@stencila/types'
import { html, LitElement } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { withTwind } from '../../twind'

import { nodeBorderColour } from './node-ui'

/**
 * A component to render the `authors` property of a node
 */
@customElement('stencila-node-authors')
@withTwind()
export class NodeAuthors extends LitElement {
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

  override firstUpdated() {
    const slot: HTMLSlotElement = this.shadowRoot.querySelector('slot')
    if (slot) {
      this.hasItems = slot.assignedElements({ flatten: true }).length !== 0
    }
  }

  override render() {
    const borderColour = nodeBorderColour(this.type)

    return html`<div class=${this.hasItems ? `block` : 'hidden'}>
      <span class="items-center flex">
        <sl-icon name="authors" library="stencila" class="pr-2"></sl-icon
        >Authors</span
      >
      <div class=${`border-b border-[${borderColour}] rounded-full my-2`}></div>
      <slot></slot>
    </div>`
  }
}
