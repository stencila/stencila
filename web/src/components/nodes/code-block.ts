import '@shoelace-style/shoelace/dist/components/menu-item/menu-item'
import '@shoelace-style/shoelace/dist/components/select/select'
import '@shoelace-style/shoelace/dist/components/tooltip/tooltip'
import { html } from 'lit'
import { customElement } from 'lit/decorators'
import { isCodeWriteable } from '../../mode'
import '../base/icon'
import '../base/tag'
import '../editors/code-editor/code-editor'
import { twSheet } from '../utils/css'
import StencilaCodeStatic from './code-static'
import './duration'
import './timestamp'

const { tw, sheet } = twSheet()

/**
 * A component representing a Stencila `CodeBlock`
 */
@customElement('stencila-code-block')
export default class StencilaCodeBlock extends StencilaCodeStatic {
  static styles = sheet.target

  static color = 'blue'

  static formats = ['markdown', 'yaml', 'json']

  render() {
    const readOnly = !isCodeWriteable()

    const toggleSelected = () => this.toggleSelected()

    return html`<div
      part="base"
      class=${tw`my-4 rounded overflow-hidden border(& ${
        StencilaCodeBlock.color
      }-200) ${this.selected ? `ring-1` : ''}`}
    >
      <div
        part="header"
        class=${tw`flex justify-between items-center bg-${StencilaCodeBlock.color}-50
                 p-1 font(mono bold) text(sm ${StencilaCodeBlock.color}-700)`}
        @mousedown=${toggleSelected}
      >
        <span class=${tw`flex items-center ml-1 mr-2`}>
          <stencila-icon class=${tw`text-base`} name="code"></stencila-icon>
          <span class=${tw`ml-2`}
            >${this.programmingLanguage.toLowerCase()}</span
          >
        </span>
        <span class=${tw`flex items-center`}>
          ${this.renderLanguageMenu(tw)} ${this.renderExpandButton(tw)}
        </span>
      </div>

      <div
        class=${this.isExpanded
          ? tw`border(t ${StencilaCodeBlock.color}-200)`
          : tw`hidden`}
      >
        <stencila-code-editor
          language=${this.programmingLanguage}
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
      </div>

      <div
        part="footer"
        class=${tw`grid justify-items-end items-center bg-${StencilaCodeBlock.color}-50
                 border(t ${StencilaCodeBlock.color}-200) p-1 text(sm ${StencilaCodeBlock.color}-700)`}
        @mousedown=${toggleSelected}
      >
        ${this.renderDownloadButton(
          StencilaCodeBlock.formats,
          StencilaCodeBlock.color
        )}
      </div>
    </div>`
  }
}
