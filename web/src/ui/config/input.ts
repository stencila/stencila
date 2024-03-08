import { css } from '@twind/core'
import { LitElement, PropertyValueMap, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'
import { Ref, createRef, ref } from 'lit/directives/ref'

import { withTwind } from '../../twind'

/**
 * UI Input field
 *
 * A shoelace styled Lit input field for our config panel.
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

  /**
   * Callback to fire when input value changes
   */
  @property()
  changeEvent: ((element: HTMLInputElement) => void) | undefined

  /**
   * Callback to fire when clearing the field
   */
  @property()
  clearEvent: (() => void) | undefined

  /**
   * Tracks the config screen's visibility. Used to determine whether to reset
   * the field or not.
   */
  @property({ type: Boolean })
  isConfigOpen: boolean = false

  /**
   * The ref used by this component.
   */
  private ref: Ref<HTMLInputElement> = createRef()

  /**
   * Using a ref for the form element so we can reset it. This is a work around
   * - wrapping the shoelace component removes our ability to directly
   * manipulate the internal input state. It does mean that each input has its
   * own form. If we add a form outside of this component in the future, we may
   * encounter nesting issues.
   */
  private formRef: Ref<HTMLFormElement> = createRef()

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
        --sl-input-height-small: 36px;
        width: 100%;
      }

      &::part(base) {
        box-shadow: 0px 0px 4px 0px rgba(0, 0, 0, 0.25) inset;
      }

      &::part(input) {
        --sl-input-spacing-small: 8px;
        background: none;
        padding: 4px var(--sl-input-spacing-small);
      }

      &::part(form-control-help-text) {
        padding: 0 var(--sl-input-spacing-small);
        color: var(--sl-input-focus-ring-color);
      }
    `

    return html`
      <form
        novalidate
        onsubmit="return false;"
        ${ref(this.formRef)}
        class="mb-5 w-full"
      >
        <sl-input
          class="${styles} w-full"
          size="small"
          value=${this.value}
          defaultValue=${this.defaultValue}
          clearable
          spellcheck=${false}
          ${ref(this.ref)}
        >
          <sl-icon
            slot="clear-icon"
            name="close-outline"
            library="stencila"
            class="fill-grey-600 text-xl hover:fill-black"
          ></sl-icon>
        </sl-input>
      </form>
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

    this.ref.value.addEventListener('sl-clear', () => {
      this.clearEvent && this.clearEvent()
    })
  }

  /**
   * Check that the config is closed, then get the form to reset (clearing any
   * fields that are dirty).
   */
  protected override update(
    changedProperties: PropertyValueMap<this> | Map<PropertyKey, unknown>
  ): void {
    super.update(changedProperties)

    if (!this.isConfigOpen && this.formRef.value) {
      this.formRef.value.reset()
    }
  }
}
