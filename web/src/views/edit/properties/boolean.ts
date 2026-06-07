/**
 * Labelled checkbox for a boolean node property.
 */
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { EDIT_PROPERTY_VALUE_CHANGE_EVENT } from './events'

@customElement('stencila-edit-boolean-property')
export class EditBooleanProperty extends LitElement {
  @property({ type: Boolean })
  checked = false

  @property()
  label = ''

  protected override createRenderRoot() {
    return this
  }

  private updateValue(event: Event) {
    this.dispatchEvent(
      new CustomEvent(EDIT_PROPERTY_VALUE_CHANGE_EVENT, {
        bubbles: true,
        composed: true,
        detail: {
          value: (event.currentTarget as HTMLInputElement).checked,
        },
      })
    )
  }

  override render() {
    return html`
      <label class="stencila-edit-node-properties-checkbox">
        <input
          type="checkbox"
          .checked=${this.checked}
          @change=${this.updateValue}
        />
        <span>${this.label}</span>
      </label>
    `
  }
}
