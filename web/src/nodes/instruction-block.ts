import { apply } from '@twind/core'
import { css, html, PropertyValues } from 'lit'
import { customElement, query, state } from 'lit/decorators.js'
import { createRef, ref, Ref } from 'lit/directives/ref'

import { documentCommandEvent } from '../clients/commands'
import { withTwind } from '../twind'
import { nodeUi } from '../ui/nodes/icons-and-colours'

import { Instruction } from './instruction'
import { SuggestionBlock } from './suggestion-block'

import '../ui/inputs/text-input'
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

  @state()
  private hasContent: boolean = false
  @state()
  private hasSuggestions: boolean = false

  private feedbackRef: Ref<HTMLInputElement> = createRef()

  /**
   * Toggles the input for revising create instructions
   */
  @state()
  private openRevisionDrawer: boolean = false

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

    this.hasContent = contentSlot !== undefined
  }

  private onSuggestionsSlotChange() {
    this.updateActiveSuggestion()
  }

  /**
   * Toggle the `isActive` class on suggestions so those that are inactive
   * are not visible.
   */
  private updateActiveSuggestion() {
    const suggestionsSlot = this.suggestionsSlot?.assignedNodes()[0] as
      | HTMLElement
      | undefined

    if (suggestionsSlot) {
      this.hasSuggestions = true
      const suggestions = Array.from(
        suggestionsSlot.children
      ) as SuggestionBlock[]

      suggestions.forEach((el, i) => {
        if (i === this.activeSuggestion) {
          el.isActive = true
          this.feedbackRef.value.value = el.feedback
        } else {
          el.isActive = false
        }
      })
    } else {
      this.hasSuggestions = false
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
    } else if (
      this.activeSuggestion === null ||
      this.activeSuggestion === undefined
    ) {
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
    } else if (
      this.activeSuggestion === null ||
      this.activeSuggestion === undefined
    ) {
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

  /**
   * Emit an event to revise the current suggestion
   */
  private revise(e: Event) {
    e.stopImmediatePropagation()

    const args = ['InstructionBlock', this.id]

    const feedback = this.feedbackRef.value.value

    if (feedback) {
      args.push(feedback)
    }

    this.dispatchEvent(
      documentCommandEvent({
        command: 'revise-node',
        args,
      })
    )
  }

  static override styles = css`
    .suggestions-container {
      position: relative;
    }

    ::slotted([slot='suggestions']) {
      display: flex;
    }
  `

  override render() {
    if (this.isWithin('StyledBlock') || this.isWithinUserChatMessage()) {
      return html`
        ${this.activeSuggestion === null || this.activeSuggestion === undefined
          ? html`<slot name="content"></slot>`
          : html`<slot name="suggestions"></slot>`}
      `
    }

    const { borderColour } = nodeUi('InstructionBlock')

    return html`<stencila-ui-block-on-demand
      type="InstructionBlock"
      header-title="${this.instructionType} Command"
      node-id=${this.id}
      depth=${this.depth}
      ancestors=${this.ancestors}
      ?noVisibleContent=${this.hasContent && !this.hasSuggestions}
      ?isRootNode=${this.root}
    >
      <span slot="header-right" class="flex">
        <stencila-ui-node-execution-commands
          type="InstructionBlock"
          node-id=${this.id}
          ?hideDropDown=${this.root}
        >
        </stencila-ui-node-execution-commands>
      </span>

      <div slot="body">
        <stencila-ui-node-execution-details
          type="InstructionBlock"
          node-id=${this.id}
          mode=${this.executionMode}
          bounds=${this.executionBounds}
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

        <slot name="model-parameters"></slot>

        <slot name="prompt"></slot>

        <div class="border-t border-[${borderColour}]">
          <slot name="message"></slot>
        </div>

        ${this.renderSuggestionsHeader()}
      </div>

      <div slot="content" class="w-full">
        <div
          class="content-container ${!(
            this.activeSuggestion === null ||
            this.activeSuggestion === undefined
          )
            ? 'hidden'
            : ''}"
        >
          <slot name="content" @slotchange=${this.onContentSlotChange}></slot>
        </div>
        <div
          class="suggestions-container ${this.activeSuggestion === null ||
          this.activeSuggestion === undefined
            ? 'hidden'
            : ''}"
        >
          <div
            style="transition: transform 0.3s ease-in-out; transform: translateX(-${this
              .activeSuggestion * 100}%)"
          >
            <slot
              name="suggestions"
              @slotchange=${this.onSuggestionsSlotChange}
            ></slot>
          </div>
        </div>
      </div>
    </stencila-ui-block-on-demand>`
  }

  private renderSuggestionsHeader() {
    const { colour, borderColour } = nodeUi('InstructionBlock')
    const suggestionsCount = this.getSuggestionsCount()

    const scrollable =
      (this.hasContent && suggestionsCount > 0) || suggestionsCount > 1

    const reviseDrawerClasses = apply([
      `bg-[${colour}]`,
      'transition-all duration-500 ease-in-out',
      this.openRevisionDrawer ? 'opacity-1 max-h-[100px]' : 'opacity-0 max-h-0',
    ])

    return html`
      <div class="font-sans">
        <div class="bg-[${borderColour}] px-3 py-2 flex justify-between">
          <span class="flex flex-row items-center gap-2">
            <stencila-ui-icon
              name="cardText"
              class="text-xl"
            ></stencila-ui-icon>
            <span class="text-sm font-bold">Suggestion</span>
          </span>

          <span class="flex flex-row items-center">
            <span class="flex flex-row items-center gap-1">
              <sl-tooltip content="Previous suggestion">
                <stencila-ui-icon-button
                  name="arrowLeftSquare"
                  @click=${(e: Event) => this.decrement(e)}
                  .disabled=${!scrollable}
                ></stencila-ui-icon-button>
              </sl-tooltip>

              <span class="text-sm"
                >${typeof this.activeSuggestion === 'number'
                  ? `${this.activeSuggestion + 1} of ${suggestionsCount}`
                  : 'Original'}</span
              >

              <sl-tooltip content="Next suggestion">
                <stencila-ui-icon-button
                  name="arrowRightSquare"
                  @click=${(e: Event) => this.increment(e)}
                  .disabled=${!scrollable}
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

            <sl-tooltip content="Revise this suggestion">
              <stencila-ui-icon-button
                name=${this.openRevisionDrawer
                  ? 'chevronDown'
                  : 'arrowClockwise'}
                class="ml-4"
                @click=${() =>
                  (this.openRevisionDrawer = !this.openRevisionDrawer)}
              ></stencila-ui-icon-button>
            </sl-tooltip>
          </span>
        </div>
        <div class=${reviseDrawerClasses}>
          <div class="flex items-center px-3 py-2">
            <ui-node-text-input
              class="w-full grow"
              card-type="InstructionBlock"
              placeholder="Describe what you want changed, or leave empty to just retry"
              .enterKeyEvent=${this.revise.bind(this)}
              ${ref(this.feedbackRef)}
            ></ui-node-text-input>
            <sl-tooltip content="Submit feedback">
              <stencila-ui-icon-button
                @click=${this.revise}
                name="arrowClockwise"
                class="ml-2"
              ></stencila-ui-icon-button>
            </sl-tooltip>
          </div>
        </div>
      </div>
    `
  }
}
