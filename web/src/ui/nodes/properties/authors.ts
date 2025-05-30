import '@shoelace-style/shoelace/dist/components/icon/icon'
import { NodeType } from '@stencila/types'
import { html, LitElement } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { withTwind } from '../../../twind'

import './generic/collapsible-details'

/**
 * A component for displaying the `authors` property of a node
 */
@customElement('stencila-ui-node-authors')
@withTwind()
export class UINodeAuthors extends LitElement {
  /**
   * The type of node that the `authors` property is on
   *
   * Used to determine the styling of this component.
   */
  @property()
  type: NodeType

  /**
   * Whether the list of authors is expanded
   */
  @property({ type: Boolean, reflect: true })
  expanded: boolean = false

  /**
   * Whether there are any authors in the list
   *
   * Used to determine if anything should be rendered.
   */
  @state()
  private hasItems = false

  /**
   * Allow the list of authors to be expanded from outside this component
   */
  public expand() {
    this.expanded = true
  }

  /**
   * Allow the list of authors to be collapsed from outside this component
   */
  public collapse() {
    this.expanded = false
  }

  override firstUpdated(changedProperties: Map<string, string | boolean>) {
    super.firstUpdated(changedProperties)

    const slot: HTMLSlotElement = this.shadowRoot.querySelector(
      'slot:not([name="provenance"])'
    )
    if (slot) {
      slot.addEventListener('slotchange', () => {
        this.hasItems = slot.assignedElements({ flatten: true }).length !== 0
      })
    }
  }

  override render() {
    return html`<stencila-ui-node-collapsible-details
      type=${this.type}
      icon-name="feather"
      header-title="Contributors"
      wrapper-css="${this.hasItems ? '' : 'hidden'}"
      ?expanded=${this.expanded}
    >
      <slot name="provenance" slot="header-content"></slot>
      <slot></slot>
    </stencila-ui-node-collapsible-details>`
  }
}
