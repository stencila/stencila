import { html } from 'lit'
import { customElement, property } from 'lit/decorators'
import { currentMode, isCodeWriteable, Mode } from '../../mode'
import StencilaInput from '../base/input'

import { twSheet } from '../utils/css'
import StencilaCodeExecutable from './code-executable'

const { tw, sheet } = twSheet()

@customElement('stencila-button')
export default class StencilaButton extends StencilaCodeExecutable {
  static styles = sheet.target

  static color = 'blue'

  static formats = ['markdown', 'yaml', 'json']

  /**
   * The `Button.name` property
   */
  @property({ reflect: true })
  name: string

  /**
   * The `Button.label` property
   */
  @property({ reflect: true })
  label: string

  /**
   * The `Button.isDisabled` property
   */
  @property({ attribute: 'is-disabled', reflect: true })
  isDisabled: string

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
      @focus=${() => this.deselect()}
      @mousedown=${(event: Event) => {
        this.deselect()
        event.stopPropagation()
      }}
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

  protected renderConditionInput() {
    const readOnly = !isCodeWriteable()

    return html`
      <div>
        <label class=${tw`mb-1`}>Enabled if</label>
      </div>
      <span class=${tw`inline-flex items-center justify-between`}>
        <stencila-code-editor
          class=${tw`min-w-0 w-full rounded overflow-hidden border(& ${StencilaButton.color}-200)
                 bg-${StencilaButton.color}-50 font-normal
                 focus:border(& ${StencilaButton.color}-400) focus:ring(2 ${StencilaButton.color}-100)`}
          language=${this.programmingLanguage}
          single-line
          line-wrapping
          no-controls
          ?read-only=${readOnly}
          ?disabled=${readOnly}
        >
          <code slot="code">${this.text}</code>
        </stencila-code-editor>

        <stencila-code-language
          class=${tw`ml-0.5 text(base blue-500)`}
          programming-language=${this.programmingLanguage}
          ?guess-language=${this.guessLanguage == 'true'}
          ?is-guessable=${true}
          ?executable-only=${true}
          ?disabled=${readOnly}
        ></stencila-code-language>
      </span>
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
        color=${StencilaButton.color}
      ></stencila-icon-button>
      <div
        class=${tw`flex flex-col gap-2 rounded border(& ${StencilaButton.color}-200)
            bg-${StencilaButton.color}-50 p-2 text(sm ${StencilaButton.color}-700)`}
      >
        ${this.renderLabelInput()} ${this.renderConditionInput()}
      </div>
    </sl-dropdown>`
  }

  protected renderButton() {
    return html`<sl-button
      style="min-width:2em"
      ?disabled=${this.isDisabled === 'true'}
      @click=${() => this.execute()}
      >${this.label}</sl-button
    >`
  }

  protected render() {
    const mode = currentMode()

    const toggleSelected = () => this.toggleSelected()

    return mode <= Mode.Interact
      ? html`<span class=${tw`inline-flex`}>${this.renderButton()}</span>`
      : html`<span
          part="base"
          class=${tw`inline-flex my-1 rounded ${this.selected ? `ring-1` : ''}`}
        >
          <span
            part="start"
            class=${tw`inline-flex items-center rounded-l overflow-hidden border(& ${StencilaButton.color}-200)
                      bg-${StencilaButton.color}-50 p-1 font(mono bold) text(sm ${StencilaButton.color}-700)`}
            @mousedown=${toggleSelected}
          >
            <span class=${tw`inline-flex items-center text-base ml-1`}>
              <stencila-icon name="button-centered"></stencila-icon>
            </span>
            <span class=${tw`ml-2 mr-2`}>button</span>
            ${this.renderNameInput()} ${this.renderSettingsDropdown()}
          </span>

          <span
            part="input"
            class=${tw`inline-flex items-center border(t b ${StencilaButton.color}-200) py-1 px-2`}
          >
            ${this.renderButton()}
          </span>

          <span
            part="end"
            class=${tw`inline-flex items-center rounded-r overflow-hidden border(& ${StencilaButton.color}-200) 
      bg-${StencilaButton.color}-50 px-1 text(sm ${StencilaButton.color}-700)`}
            @mousedown=${toggleSelected}
          >
            ${this.renderDownloadButton(
              StencilaButton.formats,
              StencilaButton.color
            )}
          </span>
        </span>`
  }
}
