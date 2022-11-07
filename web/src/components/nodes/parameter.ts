import { css, html } from 'lit'
import { customElement, property, state } from 'lit/decorators'
import { twSheet } from '../utils/css'
import StencilaExecutable from './executable'
import { StencilaValidator } from './validators'
import StencilaInput from '../base/input'
// Imports to avoid things being tree-shaken away
import './validators'
import '../base/input'
import { currentMode, isCodeWriteable, Mode } from '../../mode'

const { tw, sheet } = twSheet()

/**
 * A custom element representing a Stencila `Parameter` node
 *
 * @slot validator The `Parameter.validator` property
 */
@customElement('stencila-parameter')
export default class StencilaParameter extends StencilaExecutable {
  static styles = [
    sheet.target,
    css`
      sl-menu-item::part(label) {
        line-height: 1;
      }
    `,
  ]

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

  protected renderNameInput() {
    const readOnly = !isCodeWriteable()

    const update = (event: Event) => {
      const input = event.target as StencilaInput

      const name = input.getValue().trim()

      if (/^[a-zA-Z][a-zA-Z0-9_]*$/.test(name)) {
        input.clearError()
      } else {
        input.setError(
          'Please enter a name starting with a letter, and only containing letters, number or underscores'
        )
      }

      if (event.type == 'sl-change' && input.isValid()) {
        this.changeProperty('name', name)
      }
    }

    return html`<stencila-input
      type="text"
      size="small"
      errors="tooltip"
      class=${tw`min-w-0 w-24`}
      value=${this.name}
      ?disabled=${readOnly}
      @sl-input=${update}
      @sl-change=${update}
    ></stencila-input>`
  }

  protected renderLabelInput() {
    const readOnly = !isCodeWriteable()

    const update = (event: Event) => {
      const input = event.target as StencilaInput

      let label: string | undefined = input.getValue().trim()
      if (label.length == 0) {
        label = undefined
      }

      if (event.type == 'sl-change' && input.isValid()) {
        this.changeProperty('label', label)
      }
    }

    return html`<stencila-input
      type="text"
      label="Label"
      size="small"
      value=${this.label}
      ?disabled=${readOnly}
      @sl-input=${update}
      @sl-change=${update}
    ></stencila-input>`
  }

  protected renderLabelAndInput() {
    const inputId = `in-${Math.floor(Math.random() * 1e9)}`
    return html`<label
        for=${inputId}
        class=${tw`${this.label ? '' : 'sr-only'}`}
        >${this.label ?? this.name}</label
      >&nbsp;${this.validator?.renderInput(tw, inputId)}`
  }

  protected renderValidatorSlot() {
    return html`<slot
      name="validator"
      class=${tw`hidden`}
      @slotchange=${(event: Event) => this.onValidatorSlotChange(event)}
    ></slot>`
  }

  protected renderValidatorDropdown() {
    const readOnly = !isCodeWriteable()

    // @ts-expect-error because TS doesn't know all validator classes have an icon
    const icon = this.validator?.constructor.icon ?? 'dash-circle'

    return html`
      <sl-dropdown class=${tw`flex items-center ml-1`} ?disabled=${readOnly}>
        <stencila-icon-button
          slot="trigger"
          name=${icon}
          color="blue"
          class=${tw`text-base`}
          ?disabled=${readOnly}
        >
        </stencila-icon-button>

        <sl-menu
          @sl-select=${(event: CustomEvent) => {
            const name = event.detail.item.value

            const validator = this.validator.replaceType(name)
            this.emitOps({
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
            ([name, cls]: [string, any]) => html`<sl-menu-item
              value=${name}
              ?checked=${this.validator?.constructor == cls}
            >
              <stencila-icon slot="prefix" name=${cls.icon}></stencila-icon>
              <span class=${tw`text-sm`}> ${name} </span>
            </sl-menu-item>`
          )}
        </sl-menu>
      </sl-dropdown>
    `
  }

  protected renderSettingsDropdown() {
    const readOnly = !isCodeWriteable()

    return html`<sl-dropdown
      class=${tw`ml-1`}
      distance="10"
      placement="bottom-end"
    >
      <stencila-icon-button
        slot="trigger"
        name="three-dots-vertical"
        color=${StencilaParameter.color}
      ></stencila-icon-button>
      <div
        class=${tw`flex flex-col gap-2 rounded border(& ${StencilaParameter.color}-200)
            bg-${StencilaParameter.color}-50 p-2 text(sm ${StencilaParameter.color}-700)`}
      >
        ${this.renderLabelInput()}
        ${this.validator?.renderSettings(tw, readOnly)}
      </div>
    </sl-dropdown>`
  }

  protected render() {
    const mode = currentMode()
    return mode <= Mode.Interact
      ? html`<span class=${tw`inline-flex`}
          >${this.renderValidatorSlot()} ${this.renderLabelAndInput()}</span
        >`
      : html`<span
          part="base"
          class=${tw`inline-flex my-1 rounded ${this.selected ? `ring-1` : ''}`}
        >
          <span
            part="start"
            class=${tw`inline-flex items-center rounded-l overflow-hidden border(& ${StencilaParameter.color}-200)
                      bg-${StencilaParameter.color}-50 p-1 font(mono bold) text(sm ${StencilaParameter.color}-700)`}
          >
            <span class=${tw`inline-flex items-center text-base ml-1`}>
              <stencila-icon name="sliders"></stencila-icon>
            </span>
            <span class=${tw`ml-2 mr-2`}>par</span>
            ${this.renderNameInput()} ${this.renderValidatorSlot()}
            ${this.renderValidatorDropdown()} ${this.renderSettingsDropdown()}
          </span>

          <span
            part="input"
            class=${tw`inline-flex items-center border(t b ${StencilaParameter.color}-200) py-1 px-2`}
          >
            ${this.renderLabelAndInput()}
          </span>

          <span
            part="end"
            class=${tw`inline-flex items-center rounded-r overflow-hidden border(& ${StencilaParameter.color}-200) 
      bg-${StencilaParameter.color}-50 px-1 text(sm ${StencilaParameter.color}-700)`}
          >
            ${this.renderDownloadButton(
              StencilaParameter.formats,
              StencilaParameter.color
            )}
          </span>
        </span>`
  }
}
