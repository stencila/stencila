import { html } from 'lit'
import { customElement, property, state } from 'lit/decorators'
import { TW } from 'twind'
import { isCodeWriteable } from '../../mode'
import { Patch } from '../../types'
import { twSheet } from '../utils/css'
import StencilaCall from './call'
import StencilaParameter from './parameter'

const { tw, sheet } = twSheet()

/**
 * A component representing a Stencila `CallArgument` node
 *
 * Note that call arguments extend `Parameter` but have `code`, `programmingLanguage`,
 * and `guessLanguage` properties like `CodeExecutable` nodes. As such, this
 * component makes use of `<stencila-code-editor>` and `<stencila-code-language>`.
 *
 * @slot code The `CallArgument.code` property
 */
@customElement('stencila-call-argument')
export default class StencilaCallArgument extends StencilaParameter {
  static styles = [sheet.target]

  /**
   * The `CallArgument.programmingLanguage` property
   */
  @property({ attribute: 'programming-language', reflect: true })
  programmingLanguage: string

  /**
   * The `CallArgument.guessLanguage` property
   *
   * Because of how patching works, this property must must be a string, not a HTML boolean attribute.
   */
  @property({ attribute: 'guess-language', reflect: true })
  guessLanguage?: string

  /**
   * Whether this user is specifying the code of the argument using an input
   */
  @state()
  private useInput = false

  /**
   * Get the parent `Call` element
   */
  private getCall() {
    return this.parentElement!.parentElement! as StencilaCall
  }

  /**
   * Override of `Executable.execute` to execute the parent `Call` node by using
   * the id of the containing <stencila-call> element
   */
  protected execute() {
    this.emit('stencila-document-execute', {
      nodeId: this.getCall().id,
      ordering: 'Single',
    })
  }

  protected renderTypeIcon(tw: TW) {
    // @ts-expect-error because TS doesn't know all validator classes have an icon
    const icon = this.validator?.constructor.icon ?? 'dash-circle'
    return html`<stencila-icon name=${icon}></stencila-icon>`
  }

  protected renderCode(tw: TW) {
    return html`<div class=${tw`flex items-center w-full`}>
      ${this.renderCodeEditor(tw)} ${this.renderLanguageMenu(tw)}
    </div>`
  }

  protected renderCodeEditor(tw: TW) {
    const readOnly = !isCodeWriteable()

    return html`<stencila-code-editor
      class=${tw`min-w-0 w-full rounded overflow-hidden border(& ${StencilaCall.color}-200) 
                 focus:border(& ${StencilaCall.color}-400) focus:ring(2 ${StencilaCall.color}-100)
                 bg-${StencilaCall.color}-50 font-normal`}
      language=${this.programmingLanguage}
      single-line
      line-wrapping
      no-controls
      ?read-only=${readOnly}
      @stencila-document-patch=${(event: CustomEvent) => {
        // Emit patch using override above
        event.stopPropagation()
        this.emitPatch(event.detail.patch as Patch)
      }}
      @stencila-ctrl-enter=${() => this.execute()}
    >
      <slot name="code" slot="code"></slot>
    </stencila-code-editor>`
  }

  protected renderLanguageMenu(tw: TW) {
    return html`<stencila-code-language
      class=${tw`ml-2`}
      color=${StencilaCall.color}
      programming-language=${this.programmingLanguage}
      ?guess-language=${this.guessLanguage == 'true'}
      ?is-guessable=${true}
      ?executable-only=${true}
    ></stencila-code-language>`
  }

  protected renderInputToggle(tw: TW) {
    const toggle = () => {
      this.useInput = !this.useInput
    }

    return html`
      <stencila-icon-button
        color=${StencilaCall.color}
        name=${this.useInput ? 'code' : 'sliders'}
        @click=${toggle}
        @keypress=${toggle}
      ></stencila-icon-button>
    `
  }

  protected render() {
    return html`<div
      part="base"
      class=${tw`flex items-center justify-between whitespace-normal 
                 border(t ${StencilaCall.color}-200) bg-${StencilaCall.color}-50
                 p-1 pl-2 pr-2 font(mono) text(sm ${StencilaCall.color}-700)`}
    >
      <span part="start" class=${tw`flex items-center`}>
        ${this.renderValidatorSlot()} ${this.renderTypeIcon(tw)}
        <span class=${tw`ml-2 mr-2`}>${this.name}</span>
      </span>
      ${this.renderCode(tw)} ${this.renderInputToggle(tw)}
    </div>`
  }
}
