import '@shoelace-style/shoelace/dist/components/button/button.js'
import '@shoelace-style/shoelace/dist/components/popup/popup.js'

import { html } from 'lit'
import { customElement, state } from 'lit/decorators'

import { insertClones } from '../../clients/commands'
import { Entity } from '../../nodes/entity'
import { withTwind } from '../../twind'

import { UIBaseClass } from './mixins/ui-base-class'

@customElement('stencila-ui-nodes-selected')
@withTwind()
export class UINodesSelected extends UIBaseClass {
  /**
   * The selected nodes
   *
   * A `@state` so that when updated (due to selection change)
   * the popup changes.
   */
  @state()
  private selectedNodes: string[][] = []

  /**
   * The position of the anchor for the popup
   *
   * Does not need to be a `@state` because only ever updated
   * when `selectedNodes` is updated.
   */
  private anchorPosition = { x: 0, y: 0 }

  override connectedCallback() {
    super.connectedCallback()
    document.addEventListener(
      'selectionchange',
      this.handleSelectionChange.bind(this)
    )
  }

  override disconnectedCallback() {
    super.disconnectedCallback()
    document.removeEventListener(
      'selectionchange',
      this.handleSelectionChange.bind(this)
    )
  }

  /**
   * Handle a change in the selection
   */
  handleSelectionChange() {
    const selection = window.getSelection()
    if (!selection.rangeCount) {
      this.selectedNodes = []
      return
    }

    // Get the common ancestor of the selected range
    const range = selection.getRangeAt(0)
    let container =
      range.commonAncestorContainer.nodeType == Node.TEXT_NODE
        ? range.commonAncestorContainer.parentElement
        : (range.commonAncestorContainer as Element)

    // Walk up out of the ancestor element until we get
    // to a node type that has block content
    while (
      container &&
      !(
        container?.tagName === 'DIV' &&
        container?.getAttribute?.('slot') === 'content'
      )
    ) {
      container = container.parentElement
    }

    if (!container) {
      this.selectedNodes = []
      return
    }

    // Get selected nodes from direct children
    const selectedNodes = []
    const children = container.children
    for (const child of children) {
      if (range.intersectsNode(child) && child instanceof Entity && child.id) {
        const type = child.nodeName
        selectedNodes.push([type, child.id])
      }
    }

    if (selectedNodes.length > 0) {
      // Position anchor element near the selection
      const rect = range.getBoundingClientRect()
      this.anchorPosition = {
        x: rect.left,
        y: rect.bottom,
      }
    }

    this.selectedNodes = selectedNodes
  }

  async insertIds() {
    // Send command to insert nodes into document
    const ids = this.selectedNodes.map(([_, id]) => id)
    this.dispatchEvent(insertClones(ids))

    // Clear selection after successful insertion
    window.getSelection().removeAllRanges()

    // Clear the selected nodes so popup is hidden
    this.selectedNodes = []
  }

  override render() {
    return html`
      <div
        id="stencila-nodes-selected-anchor"
        style="
          position:absolute;
          left:${this.anchorPosition.x}px;
          top:${this.anchorPosition.y}px"
      ></div>

      <sl-popup
        anchor="stencila-nodes-selected-anchor"
        placement="bottom-start"
        distance="10"
        ?active=${this.selectedNodes.length > 0}
      >
        <div class="bg-white p-3 font-sans text-sm rounded border">
          <div>
            ${this.selectedNodes.map(
              ([type, id]) => html`<div>${type}: ${id}</div>`
            )}
          </div>

          <sl-button @click=${this.insertIds}> Insert </sl-button>
        </div>
      </sl-popup>
    `
  }
}
