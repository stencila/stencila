/**
 * Text input for a node's programming language.
 */
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { EDIT_PROPERTY_VALUE_CHANGE_EVENT } from './events'

@customElement('stencila-edit-programming-language-property')
export class EditProgrammingLanguageProperty extends LitElement {
  @property()
  value = ''

  protected override createRenderRoot() {
    return this
  }

  private updateValue(event: Event) {
    this.dispatchEvent(
      new CustomEvent(EDIT_PROPERTY_VALUE_CHANGE_EVENT, {
        bubbles: true,
        composed: true,
        detail: {
          value: (event.currentTarget as HTMLInputElement).value,
        },
      })
    )
  }

  override render() {
    return html`
      <label class="stencila-edit-node-properties-field">
        <span>Programming language</span>
        <input
          class="stencila-edit-node-properties-input"
          .value=${this.value}
          autocomplete="off"
          autocapitalize="off"
          spellcheck="false"
          placeholder="python"
          @input=${this.updateValue}
        />
      </label>
    `
  }
}
