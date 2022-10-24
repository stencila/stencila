import { html } from 'lit'
import { customElement } from 'lit/decorators'
import { apply as twApply, css } from 'twind/css'

import '@shoelace-style/shoelace/dist/components/menu-item/menu-item'
import '@shoelace-style/shoelace/dist/components/select/select'
import '@shoelace-style/shoelace/dist/components/tooltip/tooltip'

import '../base/icon'
import '../base/tag'
import '../editors/code-editor'
import { twSheet, varApply, varLocal, varUse } from '../utils/css'
import StencilaCodeExecutable from './code-executable'

const { tw, sheet } = twSheet()

@customElement('stencila-code-expression')
export default class StencilaCodeExpression extends StencilaCodeExecutable {
  static styles = sheet.target

  render() {
    return html`<span class="${tw``}">
      <stencila-code-editor
        class=${tw`min-w-0 w-full rounded overflow-hidden border(& blue-200) focus:border(& blue-400) focus:ring(2 blue-100) bg-blue-50 font-normal`}
        language=${this.programmingLanguage}
        single-line
        line-wrapping
        no-controls
        placeholder="items"
        ?read-only=${this.isReadOnly()}
        @stencila-ctrl-enter=${() => this.execute()}
      >
        <slot name="text" slot="code"></slot>
      </stencila-code-editor>

      <span part="output" class=${this._hasOutputs ? 'has-outputs' : ''}>
        <slot name="output" @slotchange=${this.onOutputsSlotChange}></slot>
      </span>
    </span>`
  }
}
