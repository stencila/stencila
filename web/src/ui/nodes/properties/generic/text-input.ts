import { NodeType } from '@stencila/types'
import { apply } from '@twind/core'
import { html, LitElement } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../../twind'
import { nodeUi } from '../../icons-and-colours'

@customElement('ui-node-text-input')
@withTwind()
export class UITextInput extends LitElement {
  @property({ type: String, attribute: 'card-type' })
  cardType: NodeType

  @property({ type: Boolean })
  readonly: boolean = false

  @property({ type: Boolean })
  disabled: boolean = false

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
    const { borderColour } = nodeUi(this.cardType)

    const baseStyles = apply([
      'w-full',
      'p-1',
      `border border-[${borderColour}] rounded-sm`,
      `outline-[${borderColour}]/50`,
      'text-sm text-gray-600',
    ])

    return html`
      <input
        class="${baseStyles} ${this.inputClasses ?? ''}"
        type="text"
        value=${this.value}
        @change=${this.handleChange}
        ?readonly=${this.readonly}
        ?disabled=${this.disabled}
      />
    `
  }
}
