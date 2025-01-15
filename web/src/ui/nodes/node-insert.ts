import { css } from '@twind/core'
import { html, LitElement } from 'lit'
import { customElement, property } from 'lit/decorators'

import { insertClones } from '../../clients/commands'
import { withTwind } from '../../twind'

@customElement('stencila-ui-node-insert')
@withTwind()
export class UINodeInsert extends LitElement {
  @property({ type: Array })
  selectedNodes: [string, string][]

  async insertIds() {
    // Send command to insert nodes into document
    const ids = this.selectedNodes.map(([_, id]) => id)
    this.dispatchEvent(insertClones(ids))

    // Clear selection after successful insertion
    window.getSelection().removeAllRanges()

    // Clear the selected nodes so popup is hidden
    this.selectedNodes = []
  }

  protected override render() {
    if (this.selectedNodes.length > 1) {
      return this.renderLarge()
    }
    return this.renderSmall()
  }

  renderLarge() {
    const tagStyles = css`
      &::part(base) {
        display: flex;
        justify-content: space-between;
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
        ${
          // show the tags if more than one id selected
          this.selectedNodes.length > 1
            ? html`<div class="flex flex-col gap-y-2">
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
              </div>`
            : ''
        }
        </div>
      </div>
    `
  }

  renderSmall() {
    return html`
      <div class="bg-brand-blue text-white font-sans text-sm rounded">
        <sl-tooltip content="Insert ${this.selectedNodes[0][0]}">
          <button class="flex p-2 items-center" @click=${this.insertIds}>
            <stencila-ui-icon
              name="boxArrowInLeft"
              class="text-lg"
            ></stencila-ui-icon>
          </button>
        <sl-tooltip>
      </div>
    `
  }
}
