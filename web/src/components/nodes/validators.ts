import '@shoelace-style/shoelace/dist/components/input/input'
import SlRange from '@shoelace-style/shoelace/dist/components/range/range'
import SlSelect from '@shoelace-style/shoelace/dist/components/select/select'
import JSON5 from 'json5'
import { css, html } from 'lit'
import { ifDefined } from 'lit-html/directives/if-defined'
import { customElement, property } from 'lit/decorators'
import { TW } from 'twind'
import { Operation } from '../../types'
import StencilaInput from '../base/input'
import StencilaEntity from './entity'
import StencilaParameter from './parameter'

/**
 * An abstract base class representing a Stencila `Validator` node
 *
 * This only exists as a fallback for when the validator of a `Parameter`
 * or something else is `None` and `<stencila-validator>` is used as a placeholder.
 */
@customElement('stencila-validator')
export class StencilaValidator extends StencilaEntity {
  /**
   * A map of validator names to classes
   */
  static types() {
    return {
      None: StencilaValidator,
      Enum: StencilaEnumValidator,
      Boolean: StencilaBooleanValidator,
      Integer: StencilaIntegerValidator,
      Number: StencilaNumberValidator,
      String: StencilaStringValidator,
      Date: StencilaDateValidator,
      Time: StencilaTimeValidator,
      DateTime: StencilaDateTimeValidator,
      Timestamp: StencilaTimestampValidator,
      Duration: StencilaDurationValidator,
    }
  }

  /**
   * The name of the icon for this type of validator
   */
  static icon = 'dash-circle'

  /**
   * Serialize an instance of the validator to a JSON object
   */
  public toJSON() {
    return { type: 'Validator' }
  }

  /**
   * Handle a change to an input for the validator
   *
   * Calls the parent `Parameter`'s `changeValue` method.
   */
  public changeValue(value: boolean | number | string) {
    const param = this.parentElement
    if (param instanceof StencilaParameter) {
      param.changeValue(value)
    } else {
      console.error(
        `Expected validator parent to be a parameter but is a ${param?.tagName}`
      )
    }
  }

  /**
   * Change a property and emit an operation representing the change
   *
   * Override that prepends `validator` to the operation address so the
   * change applies to the validator, rather than the parent node.
   */
  protected changeProperty(property: string, value: unknown) {
    if (value === null || Number.isNaN(value)) {
      value = undefined
    }

    this[property] = value

    const op: Operation =
      value === undefined
        ? {
            type: 'Remove',
            address: ['validator', property],
            items: 1,
          }
        : {
            type: 'Replace',
            address: ['validator', property],
            items: 1,
            length: 1,
            value,
          }

    return this.emitOp(op)
  }

  /**
   * Transform a validator from one type to another and emit a `Replace` operation
   *
   * Where possible, to avoid the user having to retype properties that are
   * common to validator types, will transfer properties.
   */
  public replaceType(type: string): StencilaValidator {
    const Constructor = StencilaValidator.types()[type]

    if (this.constructor === Constructor) {
      return this
    }

    if (
      this instanceof StencilaIntegerValidator &&
      Constructor === StencilaNumberValidator
    ) {
      const inst = new StencilaNumberValidator()
      inst.minimum = this.minimum
      inst.exclusiveMinimum = this.exclusiveMinimum
      inst.maximum = this.maximum
      inst.exclusiveMaximum = this.exclusiveMaximum
      inst.multipleOf = this.multipleOf
      return inst
    }

    if (
      this instanceof StencilaNumberValidator &&
      Constructor === StencilaIntegerValidator
    ) {
      const inst = new StencilaIntegerValidator()
      inst.minimum =
        this.minimum !== undefined ? Math.floor(this.minimum) : undefined
      inst.exclusiveMinimum =
        this.exclusiveMinimum !== undefined
          ? Math.floor(this.exclusiveMinimum)
          : undefined
      inst.maximum =
        this.maximum !== undefined ? Math.ceil(this.maximum) : undefined
      inst.exclusiveMaximum =
        this.exclusiveMaximum !== undefined
          ? Math.ceil(this.exclusiveMaximum)
          : undefined
      inst.multipleOf =
        this.multipleOf !== undefined ? Math.floor(this.multipleOf) : undefined
      return inst
    }

    return new Constructor()
  }

  /**
   * Render an input for the validator
   */
  public renderInput(tw: TW, id: string) {
    return html`<stencila-input
      id=${id}
      size="small"
      @sl-change=${(event: Event) => {
        const input = event.target as StencilaInput
        this.changeValue(input.getValue())
      }}
    ></stencila-input>`
  }

  /**
   * Reader inputs for the properties for the validator
   */
  public renderSettings(tw: TW, readOnly: boolean) {
    return html``
  }
}

/**
 * A custom element representing a Stencila `EnumValidator` node
 *
 * Note that this node type has no properties.
 */
@customElement('stencila-enum-validator')
export class StencilaEnumValidator extends StencilaValidator {
  static icon = 'list'

  /**
   * The `EnumValidator.values` property
   */
  @property({ type: Array, reflect: true })
  values: Array<any> = []

  public toJSON() {
    return { type: 'EnumValidator', values: this.values }
  }

  public renderInput(tw: TW, id: string) {
    const width = Math.min(
      this.values.reduce((max, value) => {
        return Math.max(max, value.toString().length)
      }, 3) + 10,
      40
    )

    return html`<sl-select
      id=${id}
      style="width:${width}ch"
      size="small"
      @sl-change=${(event: Event) => {
        const input = event.target as SlSelect
        this.changeValue(input.value as string)
      }}
      >${this.values.map(
        (value) => html`<sl-menu-item value=${value}>${value}</sl-menu-item>`
      )}</sl-select
    >`
  }

  public renderSettings(tw: TW, readOnly: boolean) {
    const update = (event: Event) => {
      const input = event.target as StencilaInput

      const value = input.getValue().trim()

      if (value.length === 0) {
        input.setError('Please enter a JSON/5 array of allowed values')
      } else {
        try {
          const values = JSON5.parse(value)
          if (!Array.isArray(values)) {
            input.setError(`Expected an array, got a ${typeof values}`)
          } else if (values.length == 0) {
            input.setError('Array should contain at least one allowed value')
          } else {
            input.clearError()
            if (event.type === 'sl-change') {
              this.changeProperty('values', values)
            }
          }
        } catch (error) {
          input.setError(`${error}`)
        }
      }
    }

    return html`<stencila-input
      label="Values"
      type="text"
      size="small"
      value=${JSON5.stringify(this.values)}
      ?disabled=${readOnly}
      @sl-input=${update}
      @sl-change=${update}
    ></stencila-input>`
  }
}

/**
 * A custom element representing a Stencila `BooleanValidator` node
 *
 * Note that this node type has no properties.
 */
@customElement('stencila-boolean-validator')
export class StencilaBooleanValidator extends StencilaValidator {
  static icon = 'boolean'

  public toJSON() {
    return { type: 'BooleanValidator' }
  }

  public renderInput(tw: TW, id: string) {
    return html`<input
      id=${id}
      type="checkbox"
      style="accent-color: #bfdbfe;"
      @change=${(event: Event) => {
        const input = event.target as HTMLInputElement
        const value = input.checked

        this.emitOp({
          type: 'Replace',
          address: ['value'],
          items: 1,
          length: 1,
          value,
        })
      }}
    />`
  }
}

/**
 * Base class for numeric validators
 */
class StencilaNumericValidator extends StencilaValidator {
  /**
   * The `minimum` property
   */
  @property({ type: Number, reflect: true })
  minimum?: number

  /**
   * The `exclusiveMinimum` property
   */
  @property({ type: Number, attribute: 'exclusive-minimum', reflect: true })
  exclusiveMinimum?: number

  /**
   * The `minimum` property
   */
  @property({ type: Number, reflect: true })
  maximum?: number

  /**
   * The `exclusiveMaximum` property
   */
  @property({ type: Number, attribute: 'exclusive-maximum', reflect: true })
  exclusiveMaximum?: number

  /**
   * The `multipleOf` property
   */
  @property({ type: Number, attribute: 'multiple-of', reflect: true })
  multipleOf?: number

  public toJSON() {
    return {
      type: 'Validator',
      minimum: this.minimum,
      exclusiveMinimum: this.exclusiveMinimum,
      maximum: this.maximum,
      exclusiveMaximum: this.exclusiveMaximum,
      multipleOf: this.multipleOf,
    }
  }

  protected getDefaultStep(min: number, max: number): number {
    return 1
  }

  protected getInputPattern() {
    return '[0-9]*'
  }

  protected getInputDesc() {
    return 'a number'
  }

  protected parseValue(value: string) {
    return parseFloat(value)
  }

  public renderInput(tw: TW, id: string) {
    const min = this.minimum ?? this.exclusiveMinimum
    const max = this.maximum ?? this.exclusiveMaximum
    const step = this.multipleOf ?? this.getDefaultStep(min ?? 0, max ?? 1)

    const inputMin =
      this.exclusiveMinimum !== undefined
        ? this.exclusiveMinimum + step
        : this.minimum
    const inputMax =
      this.exclusiveMaximum !== undefined
        ? this.exclusiveMaximum - step
        : this.maximum

    const update = (event: Event) => {
      const input = event.target as StencilaInput
      const value = input.getValueAsNumber()

      if (this.minimum && value < this.minimum) {
        input.setError(
          `Please enter ${this.getInputDesc()} greater than or equal to ${
            this.minimum
          }`
        )
      } else if (this.exclusiveMinimum && value <= this.exclusiveMinimum) {
        input.setError(
          `Please enter ${this.getInputDesc()} greater than ${
            this.exclusiveMinimum
          }`
        )
      } else if (this.maximum && value > this.maximum) {
        input.setError(
          `Please enter ${this.getInputDesc()} less than or equal to ${
            this.maximum
          }`
        )
      } else if (this.exclusiveMaximum && value >= this.exclusiveMaximum) {
        input.setError(
          `Please enter ${this.getInputDesc()} less than ${
            this.exclusiveMaximum
          }`
        )
      } else if (this.multipleOf && value % this.multipleOf !== 0) {
        input.setError(
          `Please enter ${this.getInputDesc()} that is a multiple of ${
            this.multipleOf
          }`
        )
      } else {
        input.clearError()
      }

      if (event.type === 'sl-change' && input.isValid()) {
        this.changeValue(value)
      }
    }

    return inputMin !== undefined && inputMax !== undefined
      ? html`<sl-range
          id=${id}
          min=${inputMin}
          max=${inputMax}
          step=${step}
          style="--track-color-active: #2563eb; --track-color-inactive: #bfdbfe;"
          @sl-change=${(event: Event) => {
            const input = event.target as SlRange
            this.changeValue(input.value)
          }}
        ></sl-range>`
      : html`<stencila-input
          id=${id}
          type="number"
          size="small"
          errors="tooltip"
          min=${inputMin}
          max=${inputMax}
          step=${step}
          @sl-input=${update}
          @sl-change=${update}
        ></stencila-input>`
  }

  protected renderMinimumInput(tw: TW, readOnly: boolean) {
    const update = (event: Event) => {
      const input = event.target as StencilaInput

      const value = input.getValue().trim()

      let min: number | undefined
      if (value.length == 0) {
        input.clearError()
        min = undefined
      } else {
        min = this.parseValue(value)
        if (Number.isNaN(min)) {
          input.setError(`Please enter ${this.getInputDesc()}`)
        } else {
          input.clearError()
        }
      }

      if (event.type === 'sl-change' && input.isValid()) {
        if (this.exclusiveMinimum !== undefined) {
          this.changeProperty('exclusiveMinimum', min)
        } else {
          this.changeProperty('minimum', min)
        }
      }
    }

    return html` <div>
      <div class=${tw`flex items-center justify-between`}>
        <label class=${tw`mb-1`}>Minimum</label>
        <span>
          <span class=${tw`text-xs mr-1`}>Exclusive</span>
          <input
            type="checkbox"
            style="accent-color: #bfdbfe;"
            ?checked=${this.exclusiveMinimum !== undefined}
            ?disabled=${readOnly}
            @change=${(event: Event) => {
              const input = event.target as HTMLInputElement
              if (input.checked) {
                this.changeProperty('exclusiveMinimum', this.minimum ?? 0)
                this.changeProperty('minimum', undefined)
              } else {
                this.changeProperty('minimum', this.exclusiveMinimum ?? 0)
                this.changeProperty('exclusiveMinimum', undefined)
              }
            }}
          />
        </span>
      </div>
      <div class=${tw`flex items-center`}>
        <stencila-input
          type="text"
          size="small"
          inputmode="numeric"
          pattern=${this.getInputPattern()}
          value=${ifDefined(this.minimum ?? this.exclusiveMinimum)}
          ?disabled=${readOnly}
          @sl-input=${update}
          @sl-change=${update}
        ></stencila-input>
      </div>
    </div>`
  }

  protected renderMaximumInput(tw: TW, readOnly: boolean) {
    const update = (event: Event) => {
      const input = event.target as StencilaInput

      const value = input.getValue().trim()

      let max: number | undefined
      if (value.length == 0) {
        input.clearError()
        max = undefined
      } else {
        max = this.parseValue(value)
        if (Number.isNaN(max)) {
          input.setError(`Please enter ${this.getInputDesc()}`)
        } else {
          input.clearError()
        }
      }

      if (event.type === 'sl-change' && input.isValid()) {
        if (this.exclusiveMaximum !== undefined) {
          this.changeProperty('exclusiveMaximum', max)
        } else {
          this.changeProperty('maximum', max)
        }
      }
    }

    return html` <div>
      <div class=${tw`flex items-center justify-between`}>
        <label class=${tw`mb-1`}>Maximum</label>
        <span>
          <span class=${tw`text-xs mr-1`}>Exclusive</span>
          <input
            type="checkbox"
            style="accent-color: #bfdbfe;"
            ?checked=${this.exclusiveMaximum !== undefined}
            ?disabled=${readOnly}
            @change=${(event: Event) => {
              const input = event.target as HTMLInputElement
              if (input.checked) {
                this.changeProperty('exclusiveMaximum', this.maximum ?? 100)
                this.changeProperty('maximum', undefined)
              } else {
                this.changeProperty('maximum', this.exclusiveMaximum ?? 100)
                this.changeProperty('exclusiveMaximum', undefined)
              }
            }}
          />
        </span>
      </div>
      <div class=${tw`flex items-center`}>
        <stencila-input
          type="text"
          size="small"
          inputmode="numeric"
          pattern=${this.getInputPattern()}
          value=${ifDefined(this.maximum ?? this.exclusiveMaximum)}
          ?disabled=${readOnly}
          @sl-input=${update}
          @sl-change=${update}
        ></stencila-input>
      </div>
    </div>`
  }

  protected renderMultipleOfInput(tw: TW, readOnly: boolean) {
    const update = (event: Event) => {
      const input = event.target as StencilaInput

      const value = input.getValue().trim()

      let multipleOf: number | undefined
      if (value.length == 0) {
        input.clearError()
        multipleOf = undefined
      } else {
        multipleOf = this.parseValue(value)
        if (Number.isNaN(multipleOf)) {
          input.setError(`Please enter ${this.getInputDesc()}`)
        } else if (multipleOf <= 0) {
          input.setError(
            `Please enter ${this.getInputDesc()} greater than zero`
          )
        } else {
          input.clearError()
        }
      }

      if (event.type === 'sl-change' && input.isValid()) {
        this.changeProperty('multipleOf', multipleOf)
      }
    }

    return html`<stencila-input
      label="Multiple of"
      type="text"
      size="small"
      inputmode="numeric"
      pattern=${this.getInputPattern()}
      value=${ifDefined(this.multipleOf)}
      ?disabled=${readOnly}
      @sl-input=${update}
      @sl-change=${update}
    ></stencila-input>`
  }

  public renderSettings(tw: TW, readOnly: boolean) {
    return html`
      ${this.renderMinimumInput(tw, readOnly)}
      ${this.renderMaximumInput(tw, readOnly)}
      ${this.renderMultipleOfInput(tw, readOnly)}
    `
  }
}

/**
 * A custom element representing a Stencila `IntegerValidator` node
 */
@customElement('stencila-integer-validator')
export class StencilaIntegerValidator extends StencilaNumericValidator {
  static icon = 'integer'

  public toJSON() {
    return {
      ...super.toJSON(),
      type: 'IntegerValidator',
    }
  }

  protected getInputDesc() {
    return 'an integer'
  }

  protected parseValue(value: string) {
    return parseInt(value)
  }
}

/**
 * A custom element representing a Stencila `NumberValidator` node
 */
@customElement('stencila-number-validator')
export class StencilaNumberValidator extends StencilaNumericValidator {
  static icon = 'number'

  public toJSON() {
    return {
      ...super.toJSON(),
      type: 'NumberValidator',
    }
  }

  protected getDefaultStep(min: number, max: number): number {
    return (max - min) * 0.01
  }
}

/**
 * A custom element representing a Stencila `StringValidator` node
 */
@customElement('stencila-string-validator')
export class StencilaStringValidator extends StencilaValidator {
  static icon = 'string'

  /**
   * The `StringValidator.minLength` property
   */
  @property({ type: Number, attribute: 'min-length', reflect: true })
  minLength?: number

  /**
   * The `StringValidator.maxLength` property
   */
  @property({ type: Number, attribute: 'max-length', reflect: true })
  maxLength?: number

  /**
   * The `StringValidator.pattern` property
   */
  @property({ reflect: true })
  pattern?: string

  public toJSON() {
    return {
      type: 'StringValidator',
      minLength: this.minLength,
      maxLength: this.maxLength,
      pattern: this.pattern,
    }
  }

  public renderInput(tw: TW, id: string) {
    const update = (event: Event) => {
      const input = event.target as StencilaInput

      const value = input.getValue()

      if (this.minLength !== undefined && value.length < this.minLength) {
        input.setError(`Please enter at least ${this.minLength} characters`)
      } else if (
        this.maxLength !== undefined &&
        value.length > this.maxLength
      ) {
        input.setError(`Please enter no more than ${this.maxLength} characters`)
      } else if (this.pattern !== undefined && this.pattern.length > 0) {
        const regex = new RegExp(`^${this.pattern}$`)
        if (!regex.test(value)) {
          input.setError(
            `Please enter characters matching the pattern: ${this.pattern}`
          )
        } else {
          input.clearError()
        }
      } else {
        input.clearError()
      }

      if (event.type === 'sl-change' && input.isValid()) {
        this.changeValue(value)
      }
    }

    return html`<stencila-input
      id=${id}
      type="text"
      size="small"
      errors="tooltip"
      minlength=${ifDefined(this.minLength)}
      maxlength=${ifDefined(this.maxLength)}
      pattern=${ifDefined(this.pattern)}
      @sl-input=${update}
      @sl-change=${update}
    ></stencila-input>`
  }

  protected renderMinLengthInput(tw: TW, readOnly: boolean) {
    const update = (event: Event) => {
      const input = event.target as StencilaInput

      const value = input.getValue().trim()

      let minLength: number | undefined
      if (value.length == 0) {
        minLength = undefined
        input.clearError()
      } else {
        minLength = parseInt(value)
        if (Number.isNaN(minLength)) {
          input.setError(`Please enter an integer`)
        } else if (minLength < 0) {
          input.setError(
            `Please enter an integer greater than or equal to zero`
          )
        } else if (this.maxLength !== undefined && minLength > this.maxLength) {
          input.setError(
            `Please enter an integer less than or equal to the maximum length`
          )
        } else {
          input.clearError()
        }
      }

      if (event.type === 'sl-change' && input.isValid()) {
        this.changeProperty('minLength', minLength)
      }
    }

    return html`<stencila-input
      label="Minimum length"
      type="text"
      size="small"
      inputmode="numeric"
      pattern="[0-9]*"
      value=${ifDefined(this.minLength)}
      ?disabled=${readOnly}
      @sl-input=${update}
      @sl-change=${update}
    ></stencila-input>`
  }

  protected renderMaxLengthInput(tw: TW, readOnly: boolean) {
    const update = (event: Event) => {
      const input = event.target as StencilaInput

      const value = input.getValue().trim()

      let maxLength: number | undefined
      if (value.length == 0) {
        maxLength = undefined
        input.clearError()
      } else {
        maxLength = parseInt(value)
        if (Number.isNaN(maxLength)) {
          input.setError(`Please enter an integer`)
        } else if (maxLength < 1) {
          input.setError(`Please enter an integer greater than or equal to one`)
        } else if (this.minLength !== undefined && maxLength < this.minLength) {
          input.setError(
            `Please enter an integer greater than or equal to the minimum length`
          )
        } else {
          input.clearError()
        }
      }

      if (event.type == 'sl-change' && input.isValid()) {
        this.changeProperty('maxLength', maxLength)
      }
    }
    return html`<stencila-input
      label="Maximum length"
      type="text"
      size="small"
      inputmode="numeric"
      pattern="[0-9]*"
      value=${ifDefined(this.maxLength)}
      ?disabled=${readOnly}
      @sl-input=${update}
      @sl-change=${update}
    ></stencila-input>`
  }

  protected renderPatternInput(tw: TW, readOnly: boolean) {
    const update = (event: Event) => {
      const input = event.target as StencilaInput

      const value = input.getValue().trim()

      let pattern: string | undefined = value
      if (pattern.length == 0) {
        pattern = undefined
        input.clearError()
      } else {
        try {
          new RegExp(pattern)
          input.clearError()
        } catch (error) {
          input.setError(`${error}`)
        }
      }

      if (event.type == 'sl-change' && input.isValid()) {
        this.changeProperty('pattern', pattern)
      }
    }

    return html`<stencila-input
      label="Pattern"
      type="text"
      size="small"
      value=${ifDefined(this.pattern)}
      ?disabled=${readOnly}
      @sl-input=${update}
      @sl-change=${update}
    ></stencila-input>`
  }

  public renderSettings(tw: TW, readOnly: boolean) {
    return html`
      ${this.renderMinLengthInput(tw, readOnly)}
      ${this.renderMaxLengthInput(tw, readOnly)}
      ${this.renderPatternInput(tw, readOnly)}
    `
  }
}

/**
 * A custom element representing a Stencila `DateValidator` node
 */
@customElement('stencila-date-validator')
export class StencilaDateValidator extends StencilaValidator {
  static icon = 'date'

  public toJSON() {
    return {
      type: 'DateValidator',
    }
  }

  public renderInput(tw: TW, id: string) {
    return html`<sl-input
      id=${id}
      type="date"
      size="small"
      @sl-change=${(event: Event) => this.changeValue(event)}
    ></sl-input>`
  }
}

/**
 * A custom element representing a Stencila `TimeValidator` node
 */
@customElement('stencila-time-validator')
export class StencilaTimeValidator extends StencilaValidator {
  static icon = 'clock'

  public toJSON() {
    return {
      type: 'TimeValidator',
    }
  }

  public renderInput(tw: TW, id: string) {
    return html`<sl-input
      id=${id}
      type="time"
      size="small"
      @sl-change=${(event: Event) => this.changeValue(event)}
    ></sl-input>`
  }
}

/**
 * A custom element representing a Stencila `DateTimeValidator` node
 */
@customElement('stencila-datetime-validator')
export class StencilaDateTimeValidator extends StencilaValidator {
  static icon = 'date-time'

  public toJSON() {
    return {
      type: 'DateTimeValidator',
    }
  }

  public renderInput(tw: TW, id: string) {
    return html`<sl-input
      id=${id}
      type="datetime-local"
      size="small"
      @sl-change=${(event: Event) => this.changeValue(event)}
    ></sl-input>`
  }
}

/**
 * A custom element representing a Stencila `TimestampValidator` node
 */
@customElement('stencila-timestamp-validator')
export class StencilaTimestampValidator extends StencilaValidator {
  static icon = 'stamp-light'

  public toJSON() {
    return {
      type: 'TimestampValidator',
    }
  }

  public renderInput(tw: TW, id: string) {
    return html`<sl-input
      id=${id}
      type="datetime-local"
      size="small"
      @sl-change=${(event: Event) => this.changeValue(event)}
    ></sl-input>`
  }
}

/**
 * A custom element representing a Stencila `DurationValidator` node
 */
@customElement('stencila-duration-validator')
export class StencilaDurationValidator extends StencilaValidator {
  static icon = 'stopwatch'

  public toJSON() {
    return {
      type: 'DurationValidator',
    }
  }
}

/**
 * A custom element representing a Stencila `ArrayValidator` node
 */
@customElement('stencila-array-validator')
export class StencilaArrayValidator extends StencilaValidator {
  static icon = 'array'

  public toJSON() {
    return {
      type: 'ArrayValidator',
    }
  }
}

/**
 * A custom element representing a Stencila `TupleValidator` node
 */
@customElement('stencila-tuple-validator')
export class StencilaTupleValidator extends StencilaValidator {
  static icon = 'tuple'

  public toJSON() {
    return {
      type: 'TupleValidator',
    }
  }
}
