import { NodeType } from '@stencila/types'
import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { withTwind } from '../twind'
import '../ui/nodes/card'
import '../ui/nodes/properties/content-placeholder'
import { nodeUi } from '../ui/nodes/icons-and-colours'

import { Executable } from './executable'

/**
 * Web component representing a Stencila Schema `IncludeBlock` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/include-block.md
 */
@customElement('stencila-include-block')
@withTwind()
export class IncludeBlock extends Executable {
  @property()
  source: string

  /**
   * Whether the block has any content
   *
   * Used to determine whether to render placeholder text if there is no
   * content for the block.
   *
   * @see this.renderContent()
   */
  @state()
  private hasContent = false

  /**
   * A mutation observer to update the `hasContent` state when
   * the `content` slot changes
   */
  private contentObserver: MutationObserver

  /**
   * Handle a change, including on initial load, of the `content` slot
   */
  private onContentSlotChange(event: Event) {
    // Get the slot element
    const contentElem = (event.target as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0]

    // Set current state
    this.hasContent = contentElem.childElementCount > 0

    // Update the state when the slot is mutated
    this.contentObserver = new MutationObserver(() => {
      this.hasContent = contentElem.childElementCount > 0
    })
    this.contentObserver.observe(contentElem, {
      childList: true,
    })
  }

  override render() {
    return html`
      <stencila-ui-block-on-demand
        type="IncludeBlock"
        depth=${this.depth}
        ancestors=${this.ancestors}
      >
        <span slot="header-right">
          <stencila-ui-node-execution-commands
            type="IncludeBlock"
            node-id=${this.id}
          >
          </stencila-ui-node-execution-commands>
        </span>

        <div slot="body">
          <stencila-ui-node-execution-details
            type="IncludeBlock"
            mode=${this.executionMode}
            .tags=${this.executionTags}
            status=${this.executionStatus}
            required=${this.executionRequired}
            count=${this.executionCount}
            ended=${this.executionEnded}
            duration=${this.executionDuration}
          >
            <slot name="execution-dependencies"></slot>
            <slot name="execution-dependants"></slot>
          </stencila-ui-node-execution-details>

          <stencila-ui-node-authors type="IncludeBlock">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>

          ${this.renderSource('IncludeBlock')}

          <slot name="execution-messages"></slot>
        </div>

        <div slot="content">${this.renderContent()}</div>
      </stencila-ui-block-on-demand>
    `
  }

  protected renderSource(nodeType: NodeType) {
    const { borderColour } = nodeUi(nodeType)

    return html`<div
      class="flex flex-row p-2 bg-[${borderColour}] border-t border-black/20"
    >
      <sl-tooltip
        content="Relative path or URL of the source document"
        placement="top-end"
      >
        <input class="flex-grow rounded-sm px-2 font-mono text-sm h-[2em]" readonly value=${this.source} />
      </sl-tooltip>

      <sl-tooltip
        content="Open source document in another tab"
        placement="top-end"
        ><a class="inline-block ml-2" href=${this.source} target="_blank">
          <sl-icon name="box-arrow-up-right"></sl-icon> </a
      ></sl-tooltip>
    </div>`
  }

  protected renderContent() {
    const styles = apply([this.hasContent ? '' : 'pt-2'])

    return html`<div class=${styles}>
      ${this.hasContent
        ? ''
        : html`<stencila-ui-node-content-placeholder></stencila-ui-node-content-placeholder>`}
      <slot name="content" @slotchange=${this.onContentSlotChange}></slot>
    </div>`
  }
}
