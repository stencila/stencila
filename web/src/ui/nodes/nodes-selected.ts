import '@shoelace-style/shoelace/dist/components/button/button.js'
import SlPopup from '@shoelace-style/shoelace/dist/components/popup/popup.js'
import { css } from '@twind/core'
import { html, PropertyValues } from 'lit'
import { customElement, query, state } from 'lit/decorators'

import { insertClones } from '../../clients/commands'
import { ChatMessage } from '../../nodes/chat-message'
import { Entity } from '../../nodes/entity'
import { withTwind } from '../../twind'

import { UIBaseClass } from './mixins/ui-base-class'

type NodeTuple = [string, string]

// type NodeEventDetails = {
//   node: NodeTuple
//   position: {
//     x: number
//     y: number
//   }
//   chatMessage: ChatMessage
// }

@customElement('stencila-ui-nodes-selected')
@withTwind()
export class UINodesSelected extends UIBaseClass {
  /**
   * Shoelace popup element
   */
  @query('sl-popup')
  popupElement: SlPopup

  /**
   * The selected nodes
   *
   * A `@state` so that when updated (due to selection change)
   * the popup changes.
   */
  @state()
  private selectedNodes: NodeTuple[] = []

  /**
   * The parent 'stencila-chat-message' element of the current selected nodes
   */
  private targetChatMessageElement: ChatMessage | null

  /**
   * The position of the anchor for the popup
   *
   * Does not need to be a `@state` because only ever updated
   * when `selectedNodes` is updated.
   */
  private anchorPosition = { x: 0, y: 0 }

  /**
   * reset the selected nodes and popup
   */
  private reset() {
    this.selectedNodes = []
    this.targetChatMessageElement = null
    this.anchorPosition = { x: 0, y: 0 }
  }

  private resetSelectionClickHandler(event: Event) {
    if (
      this.selectedNodes.length &&
      this.targetChatMessageElement &&
      !this.targetChatMessageElement.contains(event.target as Element)
    ) {
      this.reset()
    }
  }

  // private handleNodeHover(event: Event & { detail: NodeEventDetails }) {
  //   if (!this.selectedNodes.length) {
  //     const { node, position, chatMessage } = event.detail

  //     this.hoveredNode = node
  //     this.targetChatMessageElement = chatMessage
  //     this.anchorPosition = position

  //     chatMessage.addEventListener('mouseout', () => {
  //       if (this.hoveredNode) {
  //         this.reset()
  //       }
  //     })
  //   }
  // }

  /**
   * Checks the element is the right container for the selection functionality,
   *
   * Should be a div with a slot="content" attibute,
   * Must have a parent element like so: `stencila-chat-message[message-role="Model"]`.
   */
  private isTargetContainer(element: Element | null) {
    return (
      element?.tagName === 'DIV' &&
      element.getAttribute('slot') === 'content' &&
      element.parentElement?.tagName.toLowerCase() ===
        'stencila-chat-message' &&
      element.parentElement?.getAttribute('message-role') === 'Model'
    )
  }

  private getTargetChatMessage(element: Element) {
    return element.closest('stencila-chat-message') as ChatMessage
  }

  override connectedCallback() {
    super.connectedCallback()
    document.addEventListener(
      'selectionchange',
      this.handleSelectionChange.bind(this)
    )
    // close the insert popup on the escape key
    document.addEventListener('keydown', (event: KeyboardEvent) => {
      if (event.key === 'Escape' && this.popupElement.active) {
        this.selectedNodes = []
      }
    })

    window.addEventListener('click', this.resetSelectionClickHandler.bind(this))
  }

  protected override firstUpdated(_changedProperties: PropertyValues): void {}

  override disconnectedCallback() {
    super.disconnectedCallback()
    document.removeEventListener(
      'selectionchange',
      this.handleSelectionChange.bind(this)
    )
    window.removeEventListener(
      'click',
      this.resetSelectionClickHandler.bind(this)
    )
  }

  /**
   * Handle a change in the selection
   */
  handleSelectionChange() {
    const selection = window.getSelection()

    if (!selection.rangeCount || selection.isCollapsed) {
      this.selectedNodes = []
      return
    }

    const range = selection.getRangeAt(0)

    // Get the common ancestor of the selected range
    const rangeAncestor =
      range.commonAncestorContainer.nodeType == Node.TEXT_NODE
        ? range.commonAncestorContainer.parentElement
        : (range.commonAncestorContainer as Element)

    // Walk up out of the ancestor element until we get
    // to a node type that has block content
    let container = rangeAncestor
    while (container && !this.isTargetContainer(container)) {
      container = container.parentElement
    }

    if (!container) {
      this.selectedNodes = []
      return
    }

    this.targetChatMessageElement = this.getTargetChatMessage(container)

    // Get selected nodes from direct children
    const selectedNodes: NodeTuple[] = []
    const children = container.children
    for (const child of children) {
      if (range.intersectsNode(child) && child instanceof Entity && child.id) {
        const type = child.nodeName
        selectedNodes.push([type, child.id])
      }
    }

    if (selectedNodes.length > 1) {
      // Position anchor element on top of
      const rect = range.getBoundingClientRect()
      this.anchorPosition = {
        x: rect.left + rect.width / 2,
        y: rect.top,
      }
      this.popupElement.reposition()
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
    const tagStyles = css`
      &::part(base) {
        display: flex;
        justify-content: space-between;
      }
    `

    const popupStyles = css`
      &::part(popup) {
        z-index: 20;
        box-shadow: 0px 0px 4px 0px rgba(0, 0, 0, 0.25);
      }
    `

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
        placement="top"
        distance="10"
        ?active=${this.selectedNodes.length > 1}
        strategy="absolute"
        class=${popupStyles}
      >
        <div class="p-3 bg-brand-blue text-white font-sans text-sm rounded">
          <div class="flex justify-center mb-2">
            <button class="flex flex-row items-center" @click=${this.insertIds}>
              <stencila-ui-icon
                name="boxArrowInLeft"
                class="text-lg mr-1"
              ></stencila-ui-icon>
              Insert
            </button>
          </div>
          <div class="flex flex-col gap-y-2">
            ${this.selectedNodes.map(
              ([type, nodeId]) => html`
                <sl-tag
                  size="small"
                  class=${tagStyles}
                  removable
                  @sl-remove=${() => {
                    this.selectedNodes = this.selectedNodes.filter(
                      ([_, id]) => id !== nodeId
                    )
                  }}
                >
                  ${type}
                </sl-tag>
              `
            )}
          </div>
        </div>
      </sl-popup>
    `
  }
}
