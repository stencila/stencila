import { apply } from '@twind/core'
import { html, PropertyValues } from 'lit'
import { customElement, state } from 'lit/decorators'
import { createRef, Ref, ref } from 'lit/directives/ref'

import { documentCommandEvent } from '../../../clients/commands'
import { withTwind } from '../../../twind'
import { UIBaseClass } from '../mixins/ui-base-class'

import '../../buttons/simple-icon'

@customElement('stencila-ui-suggestion-commands')
@withTwind()
export class UINodeSuggestionCommands extends UIBaseClass {
  /**
   * Emit a custom event to execute the document with this
   * node id and command scope
   */
  private emitEvent(
    e: Event,
    action: 'accept' | 'reject' | 'revise',
    instruction?: string
  ) {
    e.stopImmediatePropagation()
    this.dispatchEvent(
      documentCommandEvent({
        command: `${action}-node`,
        nodeType: this.type,
        nodeIds: [this.nodeId],
        instruction: action === 'revise' ? instruction : undefined,
      })
    )
  }

  /**
   * toggle the tooltip containing the input for revising instructions
   */
  @state()
  private showInstructInput: boolean = false

  /**
   * status variable for the revision process
   */
  @state()
  private reviseStatus: 'idle' | 'pending' = 'idle'

  /**
   * Ref for the revision input
   */
  private inputRef: Ref<HTMLInputElement> = createRef()

  protected override update(changedProperties: PropertyValues): void {
    super.update(changedProperties)
    if (changedProperties.has('showInstructInput')) {
      this.inputRef.value.focus()
    }
  }

  /**
   * method to explicitly hide the input if its opne
   */
  private hideInstructInput() {
    if (this.showInstructInput) {
      this.showInstructInput = false
    }
  }

  override connectedCallback(): void {
    // add a click event to the window to hide the input pop up when user clicks outside.
    super.connectedCallback()
    window.addEventListener('click', this.hideInstructInput.bind(this))
  }

  override disconnectedCallback(): void {
    super.disconnectedCallback()
    // cleanup the window event listener when component is unmounted.
    window.removeEventListener('click', this.hideInstructInput.bind(this))
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
          // stop the click behaviour of the card header parent element
          e.stopImmediatePropagation()
        }}
      >
        <sl-tooltip content="Accept suggestion">
          <stencila-ui-simple-icon-button
            name="hand-thumbs-up"
            .clickEvent=${(e: Event) => {
              this.emitEvent(e, 'accept')
            }}
          ></stencila-ui-simple-icon-button>
        </sl-tooltip>
        <sl-tooltip content="Reject suggestion">
          <stencila-ui-simple-icon-button
            name="hand-thumbs-down"
            .clickEvent=${(e: Event) => {
              this.emitEvent(e, 'reject')
            }}
          ></stencila-ui-simple-icon-button>
        </sl-tooltip>
        <sl-tooltip
          content="Revise suggestion with feedback"
          style="--show-delay: 1000ms;"
        >
          <stencila-ui-simple-icon-button
            name="arrow-repeat"
            .clickEvent=${() => {
              this.showInstructInput = !this.showInstructInput
            }}
          ></stencila-ui-simple-icon-button>
        </sl-tooltip>
        ${this.renderInstructInput()}
      </div>
    `
  }

  private renderInstructInput() {
    const containerStyles = apply([
      !this.showInstructInput && 'hidden',
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
      this.emitEvent(e, 'revise', this.inputRef.value.value)
      this.reviseStatus = 'pending'
      this.inputRef.value.value = ''
      this.showInstructInput = false
    }

    return html`
      <div class=${containerStyles} @click=${(e: Event) => e.stopPropagation()}>
        <div class="flex flex-row items-center text-sm">
          <textarea
            ${ref(this.inputRef)}
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
          <stencila-ui-simple-icon-button
            name="arrow-repeat"
            custom-classes="text-lg"
            .clickEvent=${submitRevision}
            ?disabled=${this.reviseStatus === 'pending'}
          >
          </stencila-ui-simple-icon-button>
        </div>
      </div>
    `
  }
}
