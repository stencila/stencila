import { NodeType } from '@stencila/types'
import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { patchValue, patchValueExecute } from '../clients/commands'
import { withTwind } from '../twind'
import { nodeUi } from '../ui/nodes/icons-and-colours'

import '../ui/nodes/commands/execution-commands'
import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/execution-details'
import '../ui/nodes/properties/execution-messages'
import '../ui/nodes/properties/provenance'
import '../ui/nodes/properties/content-placeholder'

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
   * Whether the include block is read-only.
   *
   * Currently always false, but in the future, this may depend on the context.
   */
  private readonly = false

  /**
   * A mutation observer to update the `hasContent` state when
   * the `content` slot changes
   */
  private contentObserver: MutationObserver

  /**
   * A timer on the <input> used to debounce changes
   */
  private inputTimer: NodeJS.Timeout

  /**
   * Send the patch event upon input change
   */
  private onInputChange(e: InputEvent) {
    const value = (e.target as HTMLInputElement).value
    if (this.inputTimer) {
      clearTimeout(this.inputTimer)
    }
    this.inputTimer = setTimeout(() => {
      this.dispatchEvent(patchValue('IncludeBlock', this.id, 'source', value))
    }, 300)
  }

  /**
   * Key press event handler to execute upon 'Ctrl+Enter'
   */
  private onKeydown(e: KeyboardEvent) {
    if (e.ctrlKey && e.key === 'Enter') {
      e.preventDefault()
      const value = (e.target as HTMLInputElement).value
      this.dispatchEvent(
        patchValueExecute('IncludeBlock', this.id, 'source', value)
      )
    }
  }

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
    if (this.isWithin('StyledBlock') || this.isWithinUserChatMessage()) {
      return this.renderContent()
    }

    if (this.isWithinModelChatMessage()) {
      return this.renderCardWithInsert()
    }

    return this.renderCard()
  }

  override renderCard() {
    const hasDocRoot = this.hasDocumentRootNode()

    return html`
      <stencila-ui-block-on-demand
        type="IncludeBlock"
        node-id=${this.id}
        depth=${this.depth}
        ?no-root=${!hasDocRoot}
      >
        <div slot="header-right">
          <stencila-ui-node-execution-commands
            type="IncludeBlock"
            node-id=${this.id}
            depth=${this.depth}
            status=${this.executionStatus}
            required=${this.executionRequired}
          >
          </stencila-ui-node-execution-commands>
        </div>

        <div slot="body">
          <stencila-ui-node-execution-details
            type="IncludeBlock"
            node-id=${this.id}
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

          <stencila-ui-node-execution-messages type="IncludeBlock">
            <slot name="execution-messages"></slot>
          </stencila-ui-node-execution-messages>

          <stencila-ui-node-authors type="IncludeBlock">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>

          ${this.renderSource('IncludeBlock')}
        </div>

        <div slot="content">${this.renderContent()}</div>
      </stencila-ui-block-on-demand>
    `
  }

  protected renderSource(nodeType: NodeType) {
    const { borderColour, colour } = nodeUi(nodeType)

    return html`
      <div
        class="flex flex-row gap-x-3 px-3 py-1.5 bg-[${colour}] border-t border-[${borderColour}]"
      >
        <span class="font-mono font-bold"
          >${nodeType === 'IncludeBlock' ? 'include' : 'call'}</span
        >

        <sl-tooltip
          content="Relative path or URL of the source document"
          placement="top-end"
        >
          <input
            class="flex-grow rounded-sm border border-[${borderColour}] px-2 font-mono h-[2em] outline-black"
            value=${this.source}
            @input=${this.onInputChange}
            @keydown=${this.onKeydown}
            ?readonly=${this.readonly}
            ?disabled=${this.readonly}
          />
        </sl-tooltip>
      </div>
    `
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
