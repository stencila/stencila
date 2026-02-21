import '@shoelace-style/shoelace/dist/components/button/button.js'
import SlPopup from '@shoelace-style/shoelace/dist/components/popup/popup.js'
import { css } from '@twind/core'
import { PropertyValues, html } from 'lit'
import { customElement, query, state } from 'lit/decorators'

import { ChatMessage } from '../../nodes/chat-message'
import { Entity } from '../../nodes/entity'
import { withTwind } from '../../twind'

import { UIBaseClass } from './mixins/ui-base-class'

import './node-insert'

type NodeTuple = [string, string]

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

  /**
   * Checks the element is the right container for the selection functionality,
   *
   * Should be a div with a slot="content" attribute,
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

    // window.addEventListener('click', this.resetSelectionClickHandler.bind(this))
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
    let container =
      range.commonAncestorContainer.nodeType == Node.TEXT_NODE
        ? range.commonAncestorContainer.parentElement
        : (range.commonAncestorContainer as Element)

    // Walk up out of the ancestor element until we get
    // to a node type that has block content
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
      // Position anchor element on top
      const rect = range.getBoundingClientRect()

      let top = rect.top

      const scrollContainer = this.closest('#chat-scroll-container')

      if (scrollContainer) {
        top = top + scrollContainer.scrollTop
      }

      this.anchorPosition = {
        x: rect.left + rect.width / 2,
        y: top,
      }
      this.popupElement.reposition()
      this.selectedNodes = selectedNodes
    }
  }

  /**
   * Handle the `stencila-node-insert` element's @sl-remove event.
   */
  private handleTagRemove(id: string) {
    const updatedSelection = this.selectedNodes.filter((node) => node[1] !== id)
    this.selectedNodes = updatedSelection
  }

  override render() {
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
          top:${this.anchorPosition.y}px;"
      ></div>

      <sl-popup
        anchor="stencila-nodes-selected-anchor"
        placement="top"
        distance="10"
        ?active=${this.selectedNodes.length > 0}
        strategy="absolute"
        class=${popupStyles}
      >
        <stencila-ui-node-insert
          .selectedNodes=${this.selectedNodes}
          .handleTagRemove=${this.handleTagRemove.bind(this)}
          ?clearOnInsert=${true}
        >
        </stencila-ui-node-insert>
      </sl-popup>
    `
  }
}
