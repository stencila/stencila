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
 * Note that call arguments extend `Parameter` but have `text`, programmingLanguage`,
 * and `guessLanguage` properties like `CodeExecutable` nodes. As such, this
 * component makes use of `<stencila-code-editor>` and `<stencila-code-language>`.
 *
 * @slot text The `CallArgument.text` property
 */
@customElement('stencila-call-argument')
export default class StencilaCallArgument extends StencilaParameter {
  static styles = sheet.target

  /**
   * The `CallArgument.programmingLanguage` property
   */
  @property({ attribute: 'programming-language', reflect: true })
  programmingLanguage?: string

  /**
   * The `CallArgument.guessLanguage` property
   *
   * Because of how patching works, this property must must be a string, not a HTML boolean attribute.
   */
  @property({ attribute: 'guess-language', reflect: true })
  guessLanguage?: string

  /**
   * Whether this user is specifying the call argument as an expression or value (with inputs)
   */
  @state()
  private useExpression = false

  /**
   * Get the parent `Call` element
   */
  private getCall() {
    return this.parentElement!.parentElement! as StencilaCall
  }

  /**
   * Get all the arguments in the parent `Call` element
   */
  private getCallArguments() {
    return [...this.parentElement!.children] as StencilaCallArgument[]
  }

  /**
   * Override of `Element.emitPatch` to make the parent `Call` node the `target` of
   * the patch (by using the id of the containing <stencila-call>) and prepending the address
   * with the relative address of this `CallArgument`
   */
  protected async emitPatch(patch: Patch) {
    const index = this.getCallArguments().indexOf(this)

    const ops = patch.ops.map((op) => {
      if (op.type === 'Move') {
        return {
          ...op,
          from: ['arguments', ...op.from],
          to: ['arguments', ...op.to],
        }
      } else {
        return {
          ...op,
          address: ['arguments', index, ...op.address],
        }
      }
    })

    return super.emitPatch({
      target: this.getCall().id,
      ops,
    })
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

  /**
   * Override to initialize `isExpression` to true if the `text` property is set
   */
  public connectedCallback(): void {
    super.connectedCallback()

    // TODO
  }

  /*
  protected renderTypeIcon(tw: TW) {
    return html`<stencila-icon name=${this.getTypeIcon()}></stencila-icon>`
  }

  protected renderLabelAndInput(tw: TW) {
    return html`<div class=${tw`${this.useExpression && 'hidden'}`}>
      <input type="text" />
    </div>`
  }
  */

  protected renderExpression(tw: TW) {
    return html`<div
      class=${tw`flex items-center w-full ${this.useExpression || 'hidden'}`}
    >
      ${this.renderTextEditor(tw)} ${this.renderLanguageMenu(tw)}
    </div>`
  }

  protected renderTextEditor(tw: TW) {
    const readOnly = !isCodeWriteable()

    return html`<stencila-code-editor
      class=${tw`min-w-0 w-full rounded overflow-hidden border(& ${StencilaCall.color}-200) 
                 focus:border(& ${StencilaCall.color}-400) focus:ring(2 ${StencilaCall.color}-100)
                 bg-${StencilaCall.color}-50 font-normal pr-1`}
      language=${this.programmingLanguage}
      single-line
      line-wrapping
      no-controls
      ?read-only=${readOnly}
      @stencila-document-patch=${(event: CustomEvent) => {
        // Emit patch using override above
        event.stopPropagation()
        this.emitPatch(event.detail as Patch)
      }}
      @stencila-ctrl-enter=${() => this.execute()}
    >
      <slot name="text" slot="code"></slot>
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

  protected renderExpressionToggle(tw: TW) {
    const toggle = () => {
      this.useExpression = !this.useExpression
    }

    return html`
      <stencila-icon-button
        color=${StencilaCall.color}
        adjust=${`${
          this.useExpression
            ? `bg-${StencilaCall.color}-200 border-${StencilaCall.color}-300 text-${StencilaCall.color}-700`
            : `text-${StencilaCall.color}-500`
        }`}
        name="code-greater-than"
        @click=${toggle}
        @keypress=${toggle}
      ></stencila-icon-button>
    `
  }

  protected render() {
    return html`<div
      part="base"
      class=${tw`flex items-center justify-between  whitespace-normal 
                 border(t ${StencilaCall.color}-200) bg-${StencilaCall.color}-50
                 p-1 pl-2 pr-2 font(mono) text(sm ${StencilaCall.color}-700)`}
    >
      <span part="start" class=${tw`flex items-center`}>
        ${'' /*this.renderTypeIcon(tw)*/}
        <span class=${tw`ml-2 mr-2`}>${this.name}</span>
        ${'' /*this.renderLabelAndInput(tw)*/} ${this.renderExpression(tw)}
      </span>
      ${this.renderExpressionToggle(tw)}
    </div>`
  }
}
