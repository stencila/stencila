import { SuggestionStatus } from '@stencila/types'
import { apply } from '@twind/core'
import { html, PropertyValues } from 'lit'
import { customElement, property, state } from 'lit/decorators'
import { createRef, Ref, ref } from 'lit/directives/ref'

import { documentCommandEvent } from '../../../clients/commands'
import { withTwind } from '../../../twind'
import { UIBaseClass } from '../mixins/ui-base-class'

import '../../buttons/icon'

@customElement('stencila-ui-suggestion-commands')
@withTwind()
export class UINodeSuggestionCommands extends UIBaseClass {
  /**
   * The id of the parent instruction
   *
   * Needed for emitting the `accept-node` command.
   */
  @property({ attribute: 'instruction-id' })
  instructionId: string

  /**
   * The current status of the suggestion
   */
  @property({ attribute: 'suggestion-status' })
  suggestionStatus?: SuggestionStatus

  /**
   * The current feedback on the suggestion
   */
  @property()
  feedback?: string

  /**
   * Ref for the revision input
   */
  private reviseInputRef: Ref<HTMLInputElement> = createRef()

  /**
   * Toggle the tooltip containing the input for the revise command
   */
  @state()
  private showReviseInput: boolean = false

  /**
   * Method to explicitly hide the revise input if its open
   */
  private hideReviseInput() {
    if (this.showReviseInput) {
      this.showReviseInput = false
    }
  }

  /**
   * Focus the revise input if it is shown
   *
   * This should only place the focus on the input if the window
   * is open because otherwise it can take away the focus from the
   * editor in VSCode.
   */
  protected override update(changedProperties: PropertyValues): void {
    super.update(changedProperties)
    if (changedProperties.has('showReviseInput') && this.showReviseInput) {
      this.reviseInputRef.value.focus()
    }
  }

  /**
   * Add a click event to the window to hide the input pop up when user clicks outside.
   */
  override connectedCallback(): void {
    super.connectedCallback()
    window.addEventListener('click', this.hideReviseInput.bind(this))
  }

  /**
   * Cleanup the window event listener when component is unmounted.
   */
  override disconnectedCallback(): void {
    super.disconnectedCallback()
    window.removeEventListener('click', this.hideReviseInput.bind(this))
  }

  /**
   * Emit a custom event to perform a command on the suggestion
   */
  private emitEvent(e: Event, command: 'accept' | 'reject' | 'revise') {
    e.stopImmediatePropagation()

    const nodeType = this.type
    const nodeIds =
      command === 'accept' ? [this.nodeId, this.instructionId] : [this.nodeId]

    if (command === 'revise') {
      if (this.feedback) {
        this.dispatchEvent(
          documentCommandEvent({
            command: 'patch-node',
            nodeType,
            nodeIds,
            nodeProperty: ['feedback', this.feedback],
          })
        )
      }
      this.dispatchEvent(
        documentCommandEvent({
          command: 'revise-node',
          nodeType,
          nodeIds,
        })
      )
    } else {
      this.dispatchEvent(
        documentCommandEvent({
          command: `${command}-node`,
          nodeType,
          nodeIds,
        })
      )
    }
  }

  protected override render() {
    const isAccepted = this.suggestionStatus === 'Accepted'
    const isRejected = this.suggestionStatus === 'Rejected'

    const containerClasses = apply([
      'relative',
      'flex flex-row gap-x-3 items-center flex-shrink-0',
      `text-${this.ui.textColour}`,
    ])

    return html`
      <div
        class=${containerClasses}
        @click=${(e: Event) => {
          // stop the click behavior of the card header parent element
          e.stopImmediatePropagation()
        }}
      >
        <sl-tooltip
          content=${isAccepted
            ? 'Suggestion has been accepted'
            : 'Accept suggestion'}
        >
          <stencila-ui-icon-button
            name=${isAccepted ? 'handThumbsUpFill' : 'handThumbsUp'}
            ?disabled=${isAccepted}
            @click=${(e: Event) => this.emitEvent(e, 'accept')}
          ></stencila-ui-icon-button>
        </sl-tooltip>
        <sl-tooltip
          content=${isRejected
            ? 'Suggestion has been rejected'
            : 'Reject suggestion'}
        >
          <stencila-ui-icon-button
            name=${isRejected ? 'handThumbsDownFill' : 'handThumbsDown'}
            ?disabled=${isRejected}
            @click=${(e: Event) => this.emitEvent(e, 'reject')}
          ></stencila-ui-icon-button>
        </sl-tooltip>
        <sl-tooltip
          content="Revise suggestion with feedback"
          style="--show-delay: 1000ms;"
          ?disabled=${this.showReviseInput}
        >
          <stencila-ui-icon-button
            name="arrowClockwise"
            @click=${() => {
              this.showReviseInput = !this.showReviseInput
            }}
          ></stencila-ui-icon-button>
        </sl-tooltip>
        
        <slot></slot>

        ${this.renderReviseInput()}
      </div>
    `
  }

  private renderReviseInput() {
    const containerStyles = apply([
      !this.showReviseInput && 'hidden',
      'absolute -top-[100%] right-0 z-50',
      'transform -translate-y-full',
      `bg-[${this.ui.borderColour}]`,
      'p-1',
      `text-[${this.ui.textColour}] text-sm`,
      'rounded shadow',
      'cursor-auto',
    ])

    const textAreaStyles = apply([
      'mr-2 px-1 rounded-sm resize-none',
      `outline-[${this.ui.textColour}]/50`,
      'text-gray-700 text-[0.85rem]',
    ])

    const submit = (e: Event) => {
      this.feedback = this.reviseInputRef.value.value
      this.emitEvent(e, 'revise')
      this.showReviseInput = false
    }

    return html`
      <div class=${containerStyles} @click=${(e: Event) => e.stopPropagation()}>
        <div class="flex flex-row items-center text-sm">
          <textarea
            ${ref(this.reviseInputRef)}
            class=${textAreaStyles}
            cols="45"
            rows="2"
            placeholder="Describe how to improve the suggestion"
            @keydown=${(e: KeyboardEvent) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault()
                submit(e)
              }
            }}
          ></textarea>
          <stencila-ui-icon-button
            name="arrowClockwise"
            class="text-lg"
            @click=${submit}
          >
          </stencila-ui-icon-button>
        </div>
      </div>
    `
  }
}
