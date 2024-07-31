import { apply } from '@twind/core'
import { html, PropertyValues } from 'lit'
import { customElement, property, state } from 'lit/decorators'
import { createRef, Ref, ref } from 'lit/directives/ref'

import { documentCommandEvent } from '../../../clients/commands'
import { withTwind } from '../../../twind'
import { UIBaseClass } from '../mixins/ui-base-class'

@customElement('stencila-ui-suggestion-commands')
@withTwind()
export class UINodeSuggestionCommands extends UIBaseClass {
  /**
   * The id of the parent instruction
   *
   * Needed for the `accept-node` event.
   */
  @property({ attribute: 'instruction-id' })
  instructionId: string

  /**
   * Toggle the tooltip containing the input for the revise command
   */
  @state()
  private showReviseInput: boolean = false

  /**
   * Status variable for the revise command
   */
  @state()
  private reviseStatus: 'idle' | 'pending' = 'idle'

  /**
   * Ref for the revision input
   */
  private reviseInputRef: Ref<HTMLInputElement> = createRef()

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
  private emitEvent(
    e: Event,
    command: 'accept' | 'reject' | 'revise',
    instruction?: string
  ) {
    e.stopImmediatePropagation()

    const nodeIds =
      command === 'accept' ? [this.nodeId, this.instructionId] : [this.nodeId]

    this.dispatchEvent(
      documentCommandEvent({
        command: `${command}-node`,
        nodeType: this.type,
        nodeIds,
        instruction: command === 'revise' ? instruction : undefined,
      })
    )
  }

  protected override render() {
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
        <sl-tooltip content="Accept suggestion">
          <sl-icon
            name="hand-thumbs-up"
            @click=${(e: Event) => {
              this.emitEvent(e, 'accept')
            }}
            class="hover:text-gray-900"
          ></sl-icon>
        </sl-tooltip>
        <sl-tooltip content="Reject suggestion">
          <sl-icon
            name="hand-thumbs-down"
            @click=${(e: Event) => {
              this.emitEvent(e, 'reject')
            }}
            class="hover:text-gray-900"
          ></sl-icon>
        </sl-tooltip>
        <sl-tooltip
          content="Revise suggestion with feedback"
          style="--show-delay: 1000ms;"
        >
          <sl-icon
            name="arrow-repeat"
            @click=${() => {
              this.showReviseInput = !this.showReviseInput
            }}
            class="hover:text-gray-900"
          ></sl-icon>
        </sl-tooltip>
        ${this.renderInstructInput()}
      </div>
    `
  }

  private renderInstructInput() {
    const containerStyles = apply([
      !this.showReviseInput && 'hidden',
      'absolute -top-[100%] right-0 z-50',
      'max-w-[24rem]',
      'transform -translate-y-full',
      `bg-[${this.ui.borderColour}]`,
      'p-1',
      `text-[${this.ui.textColour}] text-sm`,
      'rounded shadow',
      'cursor-auto',
    ])

    const submitRevision = (e: Event) => {
      this.emitEvent(e, 'revise', this.reviseInputRef.value.value)
      this.reviseStatus = 'pending'
      this.reviseInputRef.value.value = ''
      this.showReviseInput = false
    }

    return html`
      <div class=${containerStyles} @click=${(e: Event) => e.stopPropagation()}>
        <div class="flex flex-row items-center text-sm">
          <textarea
            ${ref(this.reviseInputRef)}
            class="mr-2 px-1 text-gray-800 text-xs rounded-sm resize-none outline-black"
            cols="40"
            rows="3"
            placeholder="Provide feedback or leave empty for machine generated feedback"
            ?disabled=${this.reviseStatus === 'pending'}
            @keydown=${(e: KeyboardEvent) => {
              if (e.key === 'Enter' && !e.shiftKey) {
                e.preventDefault()
                submitRevision(e)
              }
            }}
          ></textarea>
          <button
            @click=${submitRevision}
            class="flex items-center cursor-pointer hover:text-gray-500"
          >
            <sl-icon
              name="arrow-repeat"
              class="text-lg"
              ?disabled=${this.reviseStatus === 'pending'}
            ></sl-icon>
          </button>
        </div>
      </div>
    `
  }
}
