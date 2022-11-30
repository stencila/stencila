import { html } from 'lit'
import { customElement } from 'lit/decorators'

import '../editors/code-editor/code-editor'
import '../base/icon'

import { isCodeWriteable } from '../../mode'
import { twSheet } from '../utils/css'
import StencilaCodeExecutable from './code-executable'

const { tw, sheet } = twSheet()

/**
 * A component representing a Stencila `CodeChunk`
 *
 * See the Stencila Schema reference documentation for details on the
 * properties of a `CodeChunk`.
 */
@customElement('stencila-code-chunk')
export default class StencilaCodeChunk extends StencilaCodeExecutable {
  static styles = sheet.target

  static color = 'blue'

  static formats = ['markdown', 'python', 'javascript', 'r', 'yaml', 'json']

  protected renderTextEditor() {
    const readOnly = !isCodeWriteable()

    return html`<div
      class=${this.isCodeVisible && this.isExpanded
        ? tw`border(t ${StencilaCodeChunk.color}-200)`
        : tw`hidden`}
    >
      <stencila-code-editor
        language=${this.programmingLanguage}
        no-controls
        ?read-only=${readOnly}
        ?disabled=${readOnly}
        @stencila-ctrl-enter=${() => this.execute()}
        @focus=${() => this.deselect()}
        @mousedown=${(event: Event) => {
          this.deselect()
          event.stopPropagation()
        }}
      >
        <slot name="code" slot="code"></slot>
      </stencila-code-editor>
    </div>`
  }

  protected renderErrorsContainer() {
    return html`<div
      part="errors"
      class=${this.hasErrors && this.isExpanded
        ? tw`border(t ${StencilaCodeChunk.color}-200)`
        : tw`hidden`}
    >
      <slot name="errors" @slotchange=${this.onErrorsSlotChange}></slot>
    </div>`
  }

  protected renderOutputsContainer() {
    return html`<div
      part="outputs"
      class=${this.hasOutputs && this.isExpanded
        ? tw`border(t ${StencilaCodeChunk.color}-200) p-1`
        : tw`hidden`}
    >
      <slot name="outputs" @slotchange=${this.onOutputsSlotChange}></slot>
    </div>`
  }

  render() {
    const toggleSelected = () => this.toggleSelected()

    return html`<div
      part="base"
      class=${tw`my-4 rounded overflow-hidden whitespace-normal border(& ${
        StencilaCodeChunk.color
      }-200) ${this.selected ? `ring-1` : ''}`}
    >
      <div
        part="header"
        class=${tw`flex justify-between items-center bg-${StencilaCodeChunk.color}-50
                 p-1 font(mono bold) text(sm ${StencilaCodeChunk.color}-700)`}
        @mousedown=${toggleSelected}
      >
        <span class=${tw`flex items-center ml-1 mr-2`}>
          <stencila-icon
            class=${tw`text-base`}
            name="lightning"
          ></stencila-icon>
          <span class=${tw`ml-2`}
            >${this.programmingLanguage.toLowerCase()}</span
          >
        </span>
        <span class=${tw`flex items-center`}>
          ${this.renderLanguageMenu(tw)} ${this.renderViewCodeButton(tw)}
          ${this.renderExpandButton(tw)}
        </span>
      </div>

      ${this.renderTextEditor()} ${this.renderErrorsContainer()}
      ${this.renderOutputsContainer()}

      <div
        part="footer"
        class=${tw`grid justify-items-end items-center bg-${StencilaCodeChunk.color}-50
                 border(t ${StencilaCodeChunk.color}-200) p-1 text(sm ${StencilaCodeChunk.color}-700)`}
        @mousedown=${toggleSelected}
      >
        ${this.renderDownloadButton(
          StencilaCodeChunk.formats,
          StencilaCodeChunk.color
        )}
      </div>
    </div>`
  }
}
