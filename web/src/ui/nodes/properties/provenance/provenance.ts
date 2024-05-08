import '@shoelace-style/shoelace/dist/components/icon/icon'
import { NodeType, ProvenanceCategory } from '@stencila/types'
import { html, LitElement, PropertyValues } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { withTwind } from '../../../../twind'
import '../../node-card/section-header'
import '../generic/collapsible-details'
import { nodeUi } from '../../icons-and-colours'
import './provenance-category'

/**
 * A component for displaying the `provenance` property of a node.
 *
 * It is assumed that clients using this component are injecting
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

  /**
   * Add categories found in nested "provenance" slots & update the UI
   * accordingly.
   */
  @state()
  categories: ProvenanceCategory[]

  protected override firstUpdated(changedProperties: PropertyValues): void {
    super.firstUpdated(changedProperties)

    const categories: ProvenanceCategory[] = []

    const slot = this.shadowRoot.querySelector('slot')
    let assignedNodes: Element[] | undefined = undefined

    if (slot) {
      assignedNodes = slot.assignedElements({ flatten: true })
      this.hasItems = assignedNodes.length !== 0
    }

    for (const node of assignedNodes ?? []) {
      for (const child of node.children) {
        const category = child.getAttribute('provenance-category')

        if (category) {
          categories.push(category as ProvenanceCategory)
        }
      }
    }

    this.categories = categories
  }

  override render() {
    const { borderColour: headerBg } = nodeUi(this.type)

    return html`<div>
      <stencila-ui-node-card-section-header
        icon-name="handshake"
        icon-library="lucide"
        headerBg=${headerBg}
        wrapper-css=${this.hasItems ? '' : 'hidden'}
      >
        <div slot="title" class="not-italic">Provenance</div>
        <div slot="right-side" class="flex gap-x-2">
          ${this.categories?.map((category) => {
            return html`<stencila-ui-node-provenance-category
              category=${category}
            ></stencila-ui-node-provenance-category>`
          })}
        </div>
      </stencila-ui-node-card-section-header>
      <slot></slot>
    </div>`
  }
}
