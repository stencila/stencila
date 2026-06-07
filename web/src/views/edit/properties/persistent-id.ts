/**
 * Text input for a node's persistent id, with inline validation feedback.
 */
import { LitElement, html, nothing } from 'lit'
import { customElement, property } from 'lit/decorators'

import { EDIT_PROPERTY_VALUE_CHANGE_EVENT } from './events'

@customElement('stencila-edit-persistent-id-property')
export class EditPersistentIdProperty extends LitElement {
  @property()
  value = ''

  @property()
  error?: string

  /**
   * Focus the input once rendered.
   *
   * The input is several light-DOM custom elements deep inside the popover, so
   * the leaf that actually owns it focuses itself on first render rather than
   * relying on an ancestor to reach in before the input exists.
   */
  @property({ type: Boolean })
  override autofocus = false

  protected override createRenderRoot() {
    return this
  }

  protected override firstUpdated() {
    if (this.autofocus) {
      this.querySelector<HTMLInputElement>(
        '.stencila-edit-node-properties-input'
      )?.focus()
    }
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
        <span>Persistent id</span>
        <input
          class="stencila-edit-node-properties-input"
          .value=${this.value}
          autocomplete="off"
          autocapitalize="off"
          spellcheck="false"
          placeholder="setup-code"
          @input=${this.updateValue}
        />
      </label>
      ${this.error
        ? html`<div class="stencila-edit-node-properties-error" role="alert">
            ${this.error}
          </div>`
        : nothing}
    `
  }
}
