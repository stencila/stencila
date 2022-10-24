import { html } from 'lit'
import { customElement, property, state } from 'lit/decorators'
import { twSheet } from '../utils/css'
import StencilaExecutable from './executable'
import { StencilaValidator } from './validators'
// Import all validators to vavoid them being tree-shaken away
import './validators'
import { TW } from 'twind'
import { SlInput } from '@shoelace-style/shoelace'
import { sentenceCase } from 'change-case'

const { tw, sheet } = twSheet()

/**
 * A custom element representing a Stencila `Parameter` node
 *
 * @slot validator The `Parameter.validator` property
 */
@customElement('stencila-parameter')
export default class StencilaParameter extends StencilaExecutable {
  static styles = sheet.target

  static color = 'blue'

  static formats = ['markdown', 'yaml', 'json']

  /**
   * The `Parameter.name` property
   */
  @property({ reflect: true })
  name: string

  /**
   * The `Parameter.label` property
   */
  @property({ reflect: true })
  label: string

  /**
   * The `Parameter.validator` property
   *
   * Note: there should be a slot from which this property gets
   * instantiated by `onValidatorSlotChange`
   */
  @state()
  protected validator: StencilaValidator

  /**
   * An observer to update this parameter if its validator changes
   */
  private validatorObserver: MutationObserver

  /**
   * Handle a change to the validator slot
   *
   * This should get called on initial load and on changes to the validator.
   */
  protected onValidatorSlotChange(event: Event) {
    const validatorElem = (event.target as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0]

    if (validatorElem) {
      this.validator = validatorElem as StencilaValidator

      this.validatorObserver = new MutationObserver(() => {
        this.onValidatorChange()
      })

      this.validatorObserver.observe(validatorElem, {
        subtree: true,
        attributes: true,
        childList: true,
      })
    }
  }

  /**
   * When validator changes e.g. change in its attributes, request an update
   * of this component
   */
  protected onValidatorChange() {
    this.requestUpdate('validator')
  }

  protected renderIcon() {
    let name = 'dash-circle'
    if (this.validator) {
      name = this.validator.getIcon()
    }
    return html`<stencila-icon name=${name}></stencila-icon>`
  }

  protected renderInput(inputId: string) {
    return this.validator?.renderInput(tw, inputId)
  }

  protected renderValidatorSlot() {
    return html`<slot
      name="validator"
      class=${tw`hidden`}
      @slotchange=${(event: Event) => this.onValidatorSlotChange(event)}
    ></slot>`
  }

  protected renderSettingsDropdown() {
    return html`<sl-dropdown
      class=${tw`ml-1`}
      distance="10"
      placement="bottom-end"
    >
      <stencila-icon-button
        slot="trigger"
        name="gear"
        color=${StencilaParameter.color}
      ></stencila-icon-button>
      <div
        class=${tw`flex flex-col gap-2 rounded border(& ${StencilaParameter.color}-200)
                   bg-${StencilaParameter.color}-50 p-2 text(sm ${StencilaParameter.color}-900)`}
      >
        <sl-input
          type="text"
          label="Name"
          size="small"
          value=${this.name}
          @sl-change=${(event: Event) => {
            const input = event.target as SlInput
            this.name = input.value
            this.changeProperty('name')
          }}
        ></sl-input>

        <sl-input
          type="text"
          label="Label"
          size="small"
          value=${this.label}
          @sl-change=${(event: Event) => {
            const input = event.target as SlInput
            this.label = input.value
            this.changeProperty('label')
          }}
        ></sl-input>

        <div class=${tw`flex items-center justify-between`}>
          <label>Type</label>
          ${this.renderIcon()}
        </div>
        <select
          size="small"
          class=${tw`w-full rounded border(& gray-300) bg-white h-8`}
          @change=${(event: Event) => {
            const select = event.target as SlInput

            const validator = this.validator.replaceType(select.value)
            this.emitOperations({
              type: 'Replace',
              address: ['validator'],
              items: 1,
              length: 1,
              value: validator.toJSON(),
            })
            this.validator.replaceWith(validator)
            this.validator = validator
          }}
        >
          ${Object.entries(StencilaValidator.types()).map(
            ([value, cls]: [string, any]) => html`<option
              value=${value}
              ?selected=${this.validator?.constructor == cls}
            >
              ${sentenceCase(value)}
            </option>`
          )}
        </select>

      ${this.validator?.renderSettings(tw)}
    </sl-dropdown>`
  }

  protected render() {
    const inputId = `in-${Math.floor(Math.random() * 1e9)}`

    // Do not use `overflow-hidden` on the base <span> to avoid any tool tips on inputs
    // getting cut off
    return html`<span
      part="base"
      class=${tw`inline-flex items-center my-1 rounded border(& ${StencilaParameter.color}-200)
                 bg-${StencilaParameter.color}-50 py-1 px-1
                 font(mono) text(sm ${StencilaParameter.color}-700)`}
    >
      <span class=${tw`inline-flex items-center ml-1`}>
        ${this.renderIcon()}
      </span>
      <label class=${tw`ml-2 mr-2`} for=${inputId}>${this.name}</label>
      ${this.renderInput(inputId)} ${this.renderValidatorSlot()}
      ${this.renderSettingsDropdown()}
      ${this.renderEntityDownload(
        StencilaParameter.formats,
        StencilaParameter.color
      )}
    </span>`
  }
}
