import { NodeType } from '@stencila/types'
import { apply } from '@twind/core'
import { html, LitElement } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { withTwind } from '../../twind'
import { nodeUi } from '../nodes/icons-and-colours'

/**
 * Basic stencila text input element, for stencila node cards.
 *  - to use add a ref object to the element.
 *  - the input value can be accessed with the public `value` property
 */
@customElement('ui-node-text-input')
@withTwind()
export class UITextInput extends LitElement {
  @property({ type: String, attribute: 'card-type' })
  cardType: NodeType

  @property({ type: String })
  placeholder: string

  @property({ type: Boolean })
  readonly: boolean = false

  @property({ type: Boolean })
  disabled: boolean = false

  /**
   * Event to be fired when the enter key is pressed
   * - no event happens if this is not defined
   */
  @property({ type: Function })
  enterKeyEvent?: (e: Event) => void

  /**
   * Allows the input value of the to be accessed from the host object
   */
  @state()
  public value: string

  /**
   * Additional twind classes to apply to the input element
   */
  @property({ type: String, attribute: 'input-classes' })
  inputClasses?: string

  private handleChange(e: InputEvent) {
    this.value = (e.target as HTMLInputElement).value
  }

  protected override render() {
    const { borderColour, textColour } = nodeUi(this.cardType)

    const baseStyles = apply([
      'w-full',
      'p-1',
      `border border-[${borderColour}] rounded-sm`,
      `outline-[${borderColour}]/50`,
      `text-sm text-[${textColour}] placeholder-[${textColour}]/50`,
    ])

    return html`
      <input
        class="${baseStyles} ${this.inputClasses ?? ''}"
        type="text"
        .value=${this.value ?? ''}
        @keydown=${this.enterKeyEvent
          ? (e: KeyboardEvent) => {
              if (e.key === 'Enter') {
                this.enterKeyEvent(e)
              }
            }
          : undefined}
        placeholder=${this.placeholder}
        @input=${this.handleChange}
        ?readonly=${this.readonly}
        ?disabled=${this.disabled}
      />
    `
  }
}
