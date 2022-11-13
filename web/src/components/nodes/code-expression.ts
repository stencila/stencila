import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { isCodeWriteable } from '../../mode'
import { twSheet } from '../utils/css'
import StencilaCodeExecutable from './code-executable'

import '../editors/code-editor/code-editor'

const { tw, sheet } = twSheet()

/**
 * A component representing a Stencila `CodeExpression` node
 */
@customElement('stencila-code-expression')
export default class StencilaCodeExpression extends StencilaCodeExecutable {
  static styles = sheet.target

  static color = 'blue'

  static formats = ['markdown', 'yaml', 'json']

  protected renderTextEditor() {
    const readOnly = !isCodeWriteable()

    return html`<stencila-code-editor
      class=${tw`min-w-0 w-full rounded overflow-hidden
                 border(& ${StencilaCodeExpression.color}-200) bg-${StencilaCodeExpression.color}-50
                 font-normal text-base
                 focus:border(& ${StencilaCodeExpression.color}-400) focus:ring(2 ${StencilaCodeExpression.color}-100)`}
      language=${this.programmingLanguage}
      single-line
      line-wrapping
      no-controls
      ?read-only=${readOnly}
      ?disabled=${readOnly}
      @focus=${() => this.deselect()}
      @mousedown=${(event: Event) => {
        this.deselect()
        event.stopPropagation()
      }}
      @stencila-ctrl-enter=${() => this.execute()}
    >
      <slot name="text" slot="code"></slot>
    </stencila-code-editor>`
  }

  protected renderErrorsContainer() {
    return html`<span
      part="errors"
      class=${this.hasErrors
        ? tw`bg-red-50 border(l ${StencilaCodeExpression.color}-200) pt-1`
        : tw`hidden`}
    >
      <slot name="errors" @slotchange=${this.onErrorsSlotChange}></slot>
    </span>`
  }

  protected renderOutputContainer() {
    return html`<span
      part="output"
      class=${tw`inline-flex border(l ${StencilaCodeExpression.color}-200) py-1 px-2`}
    >
      <slot name="output" @slotchange=${this.onOutputsSlotChange}></slot>
    </span>`
  }

  render() {
    const toggleSelected = () => this.toggleSelected()

    return html`<span
      part="base"
      class=${tw`inline-flex rounded overflow-hidden whitespace-normal border(& ${
        StencilaCodeExpression.color
      }-200) ${this.selected ? `ring-1` : ''}`}
    >
      <span
        part="start"
        class=${tw`inline-flex items-center bg-${StencilaCodeExpression.color}-50
                   py-0.5 px-1
                   font(mono bold) text(sm ${StencilaCodeExpression.color}-700)`}
        @mousedown=${toggleSelected}
      >
        <span class=${tw`inline-flex items-center text-base ml-1`}>
          <stencila-icon name="lightning"></stencila-icon>
        </span>
        <span class=${tw`ml-1 mr-2`}
          >${this.programmingLanguage.toLowerCase()}</span
        >
        ${this.renderTextEditor()} ${this.renderLanguageMenu(tw)}
      </span>

      ${this.renderErrorsContainer()} ${this.renderOutputContainer()}

      <span
        part="end"
        class=${tw`inline-flex items-center bg-${StencilaCodeExpression.color}-50
                      border(l ${StencilaCodeExpression.color}-200) px-1 text(sm ${StencilaCodeExpression.color}-700)`}
        @mousedown=${toggleSelected}
      >
        ${this.renderDownloadButton(
          StencilaCodeExpression.formats,
          StencilaCodeExpression.color
        )}
      </span>
    </span>`
  }
}
