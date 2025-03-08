import { css } from '@twind/core'
import { html, LitElement } from 'lit'
import { customElement, property } from 'lit/decorators'

import { insertClones } from '../../clients/commands'
import { withTwind } from '../../twind'

import { nodeUi } from './icons-and-colours'
import { tagNameToNodeType } from './node-tag-map'

/**
 * Renders the control to insert selected nodes into a document.
 */
@customElement('stencila-ui-node-insert')
@withTwind()
export class UINodeInsert extends LitElement {
  /**
   * Array of string tuples containing the selected node values [(nodeName),(nodeId)]
   */
  @property({ type: Array })
  selectedNodes: [string, string][]

  /**
   * Set to true if this element should clear its selected nodes
   * after successfully inserting.
   */
  @property({ type: Boolean })
  clearOnInsert: boolean = false

  /**
   * Handler for the `sl-remove` event in the `large` size element.
   *
   * Will take a single string arg for the `id` of the removed node
   *
   * Tags will not be removable if this property is `undefined`
   */
  @property({ type: Function })
  handleTagRemove: (nodeId: string) => void

  async insertIds() {
    // Send command to insert nodes into document
    const ids = this.selectedNodes.map(([_, id]) => id)
    this.dispatchEvent(insertClones(ids))

    // Clear selection after successful insertion
    window.getSelection().removeAllRanges()

    // Clear the selected nodes so popup is hidden
    if (this.clearOnInsert) {
      this.selectedNodes = []
    }
  }

  protected override render() {
    const tagStyles = css`
      &::part(base) {
        display: flex;
        justify-content: space-between;
        align-items: center;
      }
    `

    return html`
      <div class="p-3 bg-brand-blue text-white font-sans text-sm rounded">
        <div class="flex justify-center mb-2">
          <button class="flex flex-row items-center" @click=${this.insertIds}>
            <stencila-ui-icon
              name="boxArrowInLeft"
              class="text-lg"
            ></stencila-ui-icon>
            Insert
          </button>
        </div>
        <div class="flex flex-col gap-y-2">
          ${this.selectedNodes.map(([nodeName, id]) => {
            const nodeType = tagNameToNodeType(nodeName.toLowerCase())
            const { icon } = nodeUi(nodeType)
            return html`
              <sl-tag
                size="small"
                class=${tagStyles}
                ?removable=${!!this.handleTagRemove}
                @sl-remove=${() => this.handleTagRemove(id)}
                value=${id}
              >
                <stencila-ui-icon name=${icon} class="mr-1"></stencila-ui-icon>
                ${nodeType !== 'Null' ? nodeType : 'Node'}
              </sl-tag>
            `
          })}
        </div>
      </div>
    `
  }
}
