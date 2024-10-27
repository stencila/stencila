import { apply } from '@twind/core'
import { css, html, PropertyValues } from 'lit'
import { customElement, query } from 'lit/decorators.js'

import { documentCommandEvent } from '../clients/commands'
import { withTwind } from '../twind'
import { nodeUi } from '../ui/nodes/icons-and-colours'

import { Instruction } from './instruction'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/commands/execution-commands'
import '../ui/nodes/properties/execution-details'
import '../ui/nodes/properties/execution-messages'

/**
 * Web component representing a Stencila Schema `InstructionBlock` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/edits/instruction-block.md
 */
@customElement('stencila-instruction-block')
@withTwind()
export class InstructionBlock extends Instruction {
  @query('slot[name="content"]')
  contentSlot!: HTMLSlotElement

  @query('slot[name="suggestions"]')
  suggestionsSlot!: HTMLSlotElement

  private hasContent: boolean = false

  override updated(changedProperties: PropertyValues) {
    super.updated(changedProperties)

    if (changedProperties.has('activeSuggestion')) {
      this.updateActiveSuggestion()
    }
  }

  private onContentSlotChange() {
    const contentSlot = this.contentSlot?.assignedNodes()[0] as
      | HTMLElement
      | undefined

    this.hasContent = contentSlot != undefined
  }

  private onSuggestionsSlotChange() {
    this.updateActiveSuggestion()
  }

  /**
   * Toggle the `active` class on suggestions and apply transform to scroll
   * the active suggestion into view
   */
  private updateActiveSuggestion() {
    const suggestionsSlot = this.suggestionsSlot?.assignedNodes()[0] as
      | HTMLElement
      | undefined

    if (suggestionsSlot) {
      const transform = `translateX(-${this.activeSuggestion * 100}%)`
      suggestionsSlot.style.setProperty('transform', transform)
    }
  }

  /**
   * Get the number of suggestions for this instruction
   */
  private getSuggestionsCount(): number {
    const suggestionsSlot = this.suggestionsSlot?.assignedNodes()[0] as
      | HTMLElement
      | undefined

    return suggestionsSlot
      ? suggestionsSlot.querySelectorAll('stencila-suggestion-block').length
      : 0
  }

  /**
   * Emit an event to decrement the active suggestion
   */
  private decrement(e: Event) {
    e.stopImmediatePropagation()

    const suggestionsCount = this.getSuggestionsCount()

    if (suggestionsCount === 0) {
      // Go to original content (if any)
      this.activeSuggestion = undefined
    } else if (this.activeSuggestion === undefined) {
      // Go to last suggestion
      this.activeSuggestion = suggestionsCount - 1
    } else if (this.activeSuggestion === 0) {
      if (this.hasContent) {
        // Go to original content
        this.activeSuggestion = undefined
      } else {
        // Go to last suggestion
        this.activeSuggestion = suggestionsCount - 1
      }
    } else if (this.activeSuggestion > 0) {
      // Decrement the active suggestion
      this.activeSuggestion = this.activeSuggestion - 1
    }

    this.dispatchEvent(
      documentCommandEvent({
        command: 'patch-node',
        nodeType: 'InstructionBlock',
        nodeIds: [this.id],
        nodeProperty: ['activeSuggestion', this.activeSuggestion],
      })
    )
  }

  /**
   * Emit an event to decrement the active suggestion
   */
  private increment(e: Event) {
    e.stopImmediatePropagation()

    const suggestionsCount = this.getSuggestionsCount()

    if (suggestionsCount === 0) {
      // Go to original content (if any)
      this.activeSuggestion = undefined
    } else if (this.activeSuggestion === undefined) {
      // Go to first suggestion
      this.activeSuggestion = 0
    } else if (this.activeSuggestion >= suggestionsCount - 1) {
      if (this.hasContent) {
        // Go to original content
        this.activeSuggestion = undefined
      } else if (suggestionsCount > 0) {
        // Go to first suggestion
        this.activeSuggestion = 0
      }
    } else {
      // Increment the active suggestion
      this.activeSuggestion = this.activeSuggestion + 1
    }

    this.dispatchEvent(
      documentCommandEvent({
        command: 'patch-node',
        nodeType: 'InstructionBlock',
        nodeIds: [this.id],
        nodeProperty: ['activeSuggestion', this.activeSuggestion],
      })
    )
  }

  /**
   * Emit an event to archive the instruction
   */
  private archive(e: Event) {
    e.stopImmediatePropagation()

    this.dispatchEvent(
      documentCommandEvent({
        command: 'archive-node',
        nodeType: 'InstructionBlock',
        nodeIds: [this.id],
      })
    )
  }

  static override styles = css`
    .suggestions-container {
      position: relative;
      overflow-x: hidden;
    }

    ::slotted([slot='suggestions']) {
      display: flex;
      transition: transform 0.3s ease-in-out;
    }
  `

  override render() {
    const { borderColour } = nodeUi('InstructionBlock')

    return html`<stencila-ui-block-on-demand
      type="InstructionBlock"
      header-title="${this.instructionType} Command"
      node-id=${this.id}
      depth=${this.depth}
      ancestors=${this.ancestors}
    >
      <span slot="header-right" class="flex">
        <stencila-ui-node-execution-commands
          type="InstructionBlock"
          node-id=${this.id}
        >
        </stencila-ui-node-execution-commands>
      </span>

      <div slot="body">
        <stencila-ui-node-execution-details
          type="InstructionBlock"
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

        <stencila-ui-node-execution-messages type="InstructionBlock">
          <slot name="execution-messages"></slot>
        </stencila-ui-node-execution-messages>

        ${this.renderProperties()}

        <slot name="model"></slot>

        <div class="border-t border-[${borderColour}]">
          <slot name="message"></slot>
        </div>

        ${this.renderSuggestionsHeader()}
      </div>

      <div slot="content" class="w-full">
        <div
          class="content-container ${this.activeSuggestion !== undefined
            ? 'hidden'
            : ''}"
        >
          <slot name="content" @slotchange=${this.onContentSlotChange}></slot>
        </div>
        <div
          class="suggestions-container ${this.activeSuggestion === undefined
            ? 'hidden'
            : ''}"
        >
          <slot
            name="suggestions"
            @slotchange=${this.onSuggestionsSlotChange}
          ></slot>
        </div>
      </div>
    </stencila-ui-block-on-demand>`
  }

  /**
   * Render a ribbon style container with properties of the instruction
   */
  private renderProperties() {
    const { borderColour, colour } = nodeUi('InstructionBlock')

    const styles = apply(
      'flex flex-row items-center',
      'px-3 py-1.5',
      `bg-[${colour}]`,
      'text-xs leading-tight font-sans',
      `border-t border-[${borderColour}]`,
      'gap-x-3'
    )

    const inputStyles = apply([
      `border border-[${borderColour}] rounded-sm`,
      `outline-[${borderColour}]/50`,
      'text-sm text-gray-600',
      'ml-1 p-1',
    ])

    return html`
      <div class=${styles}>
        <span class="flex flex-row items-center grow">
          <sl-tooltip content="Specified prompt">
            <stencila-ui-icon class="text-base" name="at"></stencila-ui-icon>
            <input
              class="${inputStyles} w-[70%]"
              type="text"
              value=${this.prompt}
              readonly
              disabled
            />
          </sl-tooltip>
        </span>

        <span class="flex flex-row items-center">
          <sl-tooltip content="Number of suggestions to generate">
            <stencila-ui-icon class="text-base" name="hash"></stencila-ui-icon>
            <input
              class="${inputStyles}"
              type="number"
              min="1"
              max="10"
              value=${this.replicates ?? 1}
              readonly
              disabled
            />
          </sl-tooltip>
        </span>
      </div>
    `
  }

  private renderSuggestionsHeader() {
    const { borderColour } = nodeUi('InstructionBlock')
    const suggestionsCount = this.getSuggestionsCount()

    return html`<div
      class="border-t bg-[${borderColour}] px-3 py-2 font-sans flex justify-between"
    >
      <span class="flex flex-row items-center gap-2">
        <stencila-ui-icon name="cardText" class="text-xl"></stencila-ui-icon>
        <span class="text-sm font-bold">Suggestion</span>
      </span>

      <span class="flex flex-row items-center">
        <span class="flex flex-row items-center gap-1">
          <sl-tooltip content="Previous suggestion">
            <stencila-ui-icon-button
              name="arrowLeftSquare"
              @click=${(e: Event) => this.decrement(e)}
            ></stencila-ui-icon-button>
          </sl-tooltip>

          <span class="text-sm"
            >${this.activeSuggestion >= 0
              ? `${this.activeSuggestion + 1} of ${suggestionsCount}`
              : 'Original'}</span
          >

          <sl-tooltip content="Next suggestion">
            <stencila-ui-icon-button
              name="arrowRightSquare"
              @click=${(e: Event) => this.increment(e)}
            ></stencila-ui-icon-button>
          </sl-tooltip>
        </span>

        <sl-tooltip content="Accept this suggestion">
          <stencila-ui-icon-button
            name="checkCircle"
            class="ml-4"
            @click=${(e: Event) => this.archive(e)}
          ></stencila-ui-icon-button>
        </sl-tooltip>
      </span>
    </div>`
  }
}
