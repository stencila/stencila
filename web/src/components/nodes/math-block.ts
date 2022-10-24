import { html } from 'lit'
import { customElement } from 'lit/decorators'
import { currentMode, Mode } from '../../mode'
import { twSheet } from '../utils/css'
import StencilaMath from './math'

const { tw, sheet } = twSheet()

/**
 * A component representing a Stencila `MathBlock` node
 */
@customElement('stencila-math-block')
export default class StencilaMathBlock extends StencilaMath {
  static styles = sheet.target

  static color = 'blue'

  static formats = ['markdown', 'yaml', 'json']

  render() {
    const mode = currentMode()
    return mode < Mode.Inspect
      ? html`${this.renderMathMLSlot(tw, false)}`
      : html`<div
          part="base"
          class=${tw`my-4 rounded border(& ${StencilaMathBlock.color}-200) overflow-hidden`}
        >
          <div
            part="header"
            class=${tw`flex justify-between items-center bg-${StencilaMathBlock.color}-50
                        p-1 font(mono bold) text(sm ${StencilaMathBlock.color}-700)`}
          >
            <span class=${tw`flex items-center text-base ml-1 p-1`}>
              <stencila-icon name="math"></stencila-icon>
            </span>
            <span class=${tw`mr-2`}>math</span>
            ${this.renderTextEditor(tw, StencilaMathBlock.color)}
            ${this.renderLanguageMenu(tw, StencilaMathBlock.color)}
            ${this.renderExpandButton(tw, StencilaMathBlock.color)}
          </div>

          ${this.renderMathMLSlot(
            tw,
            false,
            `border(t ${StencilaMathBlock.color}-200) p-2 ${
              this.isExpanded || 'hidden'
            }`
          )}

          <div
            part="footer"
            class=${tw`grid justify-items-end items-center bg-${StencilaMathBlock.color}-50
                       border(t ${StencilaMathBlock.color}-200) p-1 text(sm ${StencilaMathBlock.color}-700)`}
          >
            ${this.renderEntityDownload(
              StencilaMathBlock.formats,
              StencilaMathBlock.color
            )}
          </div>
        </div>`
  }
}
