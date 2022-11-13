import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { isCodeWriteable } from '../../mode'
import { twSheet } from '../utils/css'
import StencilaCodeStatic from './code-static'

import '../editors/code-editor/code-editor'

const { tw, sheet } = twSheet()

/**
 * A component representing a Stencila `CodeFragment`
 */
@customElement('stencila-code-fragment')
export default class StencilaCodeFragment extends StencilaCodeStatic {
  static styles = sheet.target

  static color = 'blue'

  static formats = ['markdown', 'yaml', 'json']

  render() {
    const readOnly = !isCodeWriteable()

    const toggleSelected = () => this.toggleSelected()

    return html`<span
      part="base"
      class=${tw`inline-flex rounded overflow-hidden whitespace-normal border(& ${
        StencilaCodeFragment.color
      }-200) ${this.selected ? `ring-1` : ''}`}
    >
      <span
        part="start"
        class=${tw`inline-flex items-center bg-${StencilaCodeFragment.color}-50
                      border(r ${StencilaCodeFragment.color}-200) font(mono bold) text(sm ${StencilaCodeFragment.color}-700)`}
        @mousedown=${toggleSelected}
      >
        <span class=${tw`inline-flex items-center text-base ml-1`}>
          <stencila-icon name="code"></stencila-icon>
        </span>
        <span class=${tw`mx-1`}>${this.programmingLanguage.toLowerCase()}</span>
      </span>

      <stencila-code-editor
        class=${tw`min-w-0 w-full rounded overflow-hidden
                   focus:border(& blue-400) focus:ring(2 blue-100) bg-blue-50`}
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
      >
        <slot name="text" slot="code"></slot>
      </stencila-code-editor>

      <span
        part="end"
        class=${tw`inline-flex items-center bg-${StencilaCodeFragment.color}-50
                      border(l ${StencilaCodeFragment.color}-200) px-1 text(sm ${StencilaCodeFragment.color}-700)`}
        @mousedown=${toggleSelected}
      >
        ${this.renderLanguageMenu(tw)}
        ${this.renderDownloadButton(
          StencilaCodeFragment.formats,
          StencilaCodeFragment.color
        )}
      </span>
    </span>`
  }
}
