import { html } from 'lit'
import { customElement } from 'lit/decorators'
import { currentMode, Mode } from '../../mode'
import { twSheet } from '../utils/css'
import StencilaMath from './math'

const { tw, sheet } = twSheet()

/**
 * A component representing a Stencila `MathFragment` node
 */
@customElement('stencila-math-fragment')
export default class StencilaMathFragment extends StencilaMath {
  static styles = sheet.target

  render() {
    const mode = currentMode()
    return mode < Mode.Inspect
      ? html`${this.renderMathMLSlot(tw, true)}`
      : html`<span
          part="base"
          class=${tw`inline-flex rounded overflow-hidden border(& emerald-200)`}
        >
          <span
            part="start"
            class=${tw`inline-flex items-center bg-emerald-100 py-0.5 px-1 font(mono bold) text(sm emerald-800)`}
          >
            <span class=${tw`inline-flex items-center text-base ml-1`}>
              <stencila-icon name="math"></stencila-icon>
            </span>
            <span class=${tw`ml-1 mr-2 `}>math</span>
            ${this.renderTextEditor(tw)} ${this.renderLanguageMenu(tw)}
          </span>
          ${this.renderMathMLSlot(tw, true, 'py-0.5 px-2')}
        </span>`
  }
}
