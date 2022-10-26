import { SlInput } from '@shoelace-style/shoelace'
import { css, html } from 'lit'
import { ifDefined } from 'lit-html/directives/if-defined'
import { customElement, property, state } from 'lit/decorators'
import { createRef, ref, Ref } from 'lit/directives/ref'
import StencilaElement from '../utils/element'

/**
 * A user input
 *
 * This is a wrapper around `<sl-input>` that styles the input if it has an error,
 * and instead of using the browsers native error reporting, allows for different
 * error reporting depending upon the context:
 *
 * - below: small text below the input
 * - tooltip: a popup tooltip below the input
 */
@customElement('stencila-input')
export default class StencilaInput extends StencilaElement {
  static styles = css`
    sl-input.invalid::part(form-control-help-text) {
      color: var(--sl-color-danger-800);
    }
    sl-input.invalid::part(base) {
      border-color: var(--sl-color-danger-300);
    }
    sl-input.invalid:focus-within::part(base) {
      box-shadow: 0 0 0 1px var(--sl-color-danger-200);
    }
    sl-tooltip::part(base__arrow),
    sl-tooltip::part(body) {
      color: var(--sl-color-danger-50);
      background-color: var(--sl-color-danger-600);
    }
  `

  /**
   * A reference to the inner `<sl-input>` used to get its value
   */
  private inner: Ref<SlInput> = createRef()

  /**
   * Where to place error messages
   */
  @property()
  errors: 'below' | 'tooltip' = 'below'

  // The following a the properties of `<sl-input>` used by STencila
  // For more see https://shoelace.style/components/input?id=properties

  /**
   * The input's type
   */
  @property()
  type:
    | 'date'
    | 'datetime-local'
    | 'email'
    | 'number'
    | 'password'
    | 'search'
    | 'tel'
    | 'text'
    | 'time'
    | 'url' = 'text'

  /**
   * The input's size
   */
  @property()
  size: 'small' | 'medium' | 'large' = 'small'

  /**
   * The input's value
   */
  @property()
  value?: string

  /**
   * The input's label
   */
  @property()
  label?: string

  /**
   * The input's placeholder text
   */
  @property()
  placeholder?: string

  /**
   * The input mode
   */
  @property()
  inputmode?:
    | 'none'
    | 'text'
    | 'decimal'
    | 'numeric'
    | 'tel'
    | 'search'
    | 'email'
    | 'url'

  /**
   * The minimum length of input that will be considered valid
   */
  @property({ type: Number })
  minlength?: number

  /**
   * The maximum length of input that will be considered valid
   */
  @property({ type: Number })
  maxlength?: number

  /**
   * A pattern to validate input against
   */
  @property()
  pattern?: string

  /**
   * The minimum that will be considered valid
   */
  @property()
  min?: number | string

  /**
   * The maximum that will be considered valid
   */
  @property()
  max?: number | string

  /**
   * Specifies the granularity that the value must adhere to,
   * or the special value any which means no stepping is implied, allowing any numeric value.
   */
  @property({ type: Number })
  step?: number

  /**
   * Makes the input a required field
   */
  @property({ type: Boolean })
  required = false

  /**
   * Is the input disabled?
   */
  @property({ type: Boolean })
  disabled = false

  /**
   * An error message for the input
   *
   * Use `setError` and `clearError` methods to alter this property.
   */
  @state()
  private error?: string

  getValue() {
    return this.inner.value!.value
  }

  getValueAsNumber() {
    return this.inner.value!.valueAsNumber
  }

  getValueAsDate() {
    return this.inner.value!.valueAsDate
  }

  setError(error: string) {
    this.error = error
  }

  clearError() {
    this.error = undefined
  }

  isValid() {
    return this.error === undefined
  }

  render() {
    // The `style` on the tooltip content helps avoid a momentarily weird looking mini-tooltip when
    // `this.error` is empty but the tooltip is still transitioning to closed.
    return html`<sl-tooltip
      placement="bottom"
      trigger="manual"
      ?open=${this.error && this.errors === 'tooltip'}
    >
      <span slot="content" style="display:inline-block; min-width: 20em"
        >${this.error}</span
      >
      <sl-input
        ${ref(this.inner)}
        class=${this.error ? 'invalid' : ''}
        type=${this.type}
        size=${this.size}
        label=${ifDefined(this.label)}
        value=${ifDefined(this.value)}
        inputmode=${ifDefined(this.inputmode)}
        minlength=${ifDefined(this.minlength)}
        maxlength=${ifDefined(this.maxlength)}
        pattern=${ifDefined(this.pattern)}
        min=${ifDefined(this.min)}
        max=${ifDefined(this.max)}
        step=${ifDefined(this.step)}
        ?required=${this.required}
        ?disabled=${this.disabled}
      >
        <small slot="help-text"
          >${this.errors === 'below' ? this.error : ''}</small
        >
      </sl-input>
    </sl-tooltip>`
  }
}
