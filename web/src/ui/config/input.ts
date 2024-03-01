import { css } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import { Ref, createRef, ref } from 'lit/directives/ref'

import { withTwind } from '../../twind'

/**
 * UI Input field
 *
 * A shoelace styled Lit input field
 */
@customElement('stencila-ui-input-field')
@withTwind()
export class UIInput extends LitElement {
  /**
   * The value to initialize the component with.
   */
  @property()
  defaultValue: string | undefined = undefined

  /**
   * The current value of the input field.
   */
  @property()
  value: string | undefined = undefined

  @property()
  changeEvent: (element: HTMLInputElement) => void | undefined

  /**
   * The ref used by this component.
   */
  private ref: Ref<HTMLInputElement> = createRef()

  override render() {
    const styles = css`
      :host {
        width: 100%;
      }

      &::part(form-control) {
        --sl-input-border-radius-small: 3px;
        --sl-input-font-size-small: 12px;
        --sl-input-color: #999999;
        --sl-input-border-color: none;
        --sl-input-border-width: 0;
        --sl-input-border-color-focus: transparent;
        --sl-focus-ring-width: 1px;
        --sl-input-focus-ring-color: #092d77;
        --sl-input-height-small: 28px;
      }

      &::part(form-control) {
        width: 100%;
      }

      &::part(input) {
        --sl-input-spacing-small: 8px;
        padding: 4px var(--sl-input-spacing-small);
        box-shadow: 0px 0px 4px 0px rgba(0, 0, 0, 0.25) inset;
      }

      &::part(form-control-help-text) {
        padding: 0 var(--sl-input-spacing-small);
        color: var(--sl-input-focus-ring-color);
      }
    `

    return html`
      <div class="mb-5 w-full">
        <sl-input
          class="${styles} w-full"
          size="small"
          defaultValue=${this.defaultValue}
          value=${this.value}
          ${ref(this.ref)}
        ></sl-input>
      </div>
    `
  }

  /**
   * Add a change event to manage change of input field.
   */
  override firstUpdated() {
    const selfRef = this.ref.value
    this.ref.value.addEventListener('sl-input', () => {
      this.changeEvent && this.changeEvent(selfRef)
    })
  }
}
