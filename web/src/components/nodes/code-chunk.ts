import '@shoelace-style/shoelace/dist/components/menu-item/menu-item'
import '@shoelace-style/shoelace/dist/components/select/select'
import '@shoelace-style/shoelace/dist/components/tooltip/tooltip'
import { html } from 'lit'
import { customElement } from 'lit/decorators'
import '../base/icon'
import '../base/tag'
import '../editors/code-editor'
import { twSheet } from '../utils/css'
import StencilaCodeExecutable from './code-executable'
import './duration'
import './timestamp'

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

  renderTextEditor() {
    const readOnly = this.isReadOnly()

    return html` <stencila-code-editor
      part="code"
      language=${this.programmingLanguage}
      no-controls
      ?read-only=${readOnly}
      ?disabled=${readOnly}
      @stencila-ctrl-enter=${() => this.execute('Topological')}
    >
      <slot name="text" slot="code"></slot>
    </stencila-code-editor>`
  }

  renderErrorsContainer() {
    return html`<div
      part="errors"
      class=${this.hasErrors
        ? tw`border(t ${StencilaCodeChunk.color}-200)`
        : tw`hidden`}
    >
      <slot name="errors" @slotchange=${this.onErrorsSlotChange}></slot>
    </div>`
  }

  renderOutputsContainer() {
    return html`<div
      part="outputs"
      class=${this.hasOutputs
        ? tw`border(t ${StencilaCodeChunk.color}-200) p-1`
        : tw`hidden`}
    >
      <slot name="outputs" @slotchange=${this.onOutputsSlotChange}></slot>
    </div>`
  }

  render() {
    return html`<div
      part="base"
      class=${tw`my-4 rounded border(& ${StencilaCodeChunk.color}-200) overflow-hidden`}
    >
      <div
        part="header"
        class=${tw`flex justify-between items-center bg-${StencilaCodeChunk.color}-50
                 border(b ${StencilaCodeChunk.color}-200) p-1 font(mono bold) text(sm ${StencilaCodeChunk.color}-700)`}
      >
        <span class=${tw`flex items-center text-base ml-1 mr-2`}>
          <stencila-icon name="lightning"></stencila-icon>
          <span class=${tw`ml-2 mr-2`}
            >${this.programmingLanguage.toLowerCase()}</span
          >
        </span>
        ${this.renderLanguageMenu(tw)}
      </div>
      ${this.renderTextEditor()} ${this.renderErrorsContainer()}
      ${this.renderOutputsContainer()}

      <div
        part="footer"
        class=${tw`grid justify-items-end items-center bg-${StencilaCodeChunk.color}-50
                 border(t ${StencilaCodeChunk.color}-200) p-1 text(sm ${StencilaCodeChunk.color}-700)`}
      >
        ${this.renderEntityDownload(
          StencilaCodeChunk.formats,
          StencilaCodeChunk.color
        )}
      </div>
    </div>`
  }
}
