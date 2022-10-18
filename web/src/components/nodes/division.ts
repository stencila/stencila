import { html } from 'lit'
import { customElement } from 'lit/decorators'
import { TW } from 'twind'
import { currentMode, Mode } from '../../mode'
import { twSheet } from '../utils/css'
import StencilaStyled from './styled'

const { tw, sheet } = twSheet()

/**
 * A component representing a Stencila `Division` node
 */
@customElement('stencila-division')
export default class StencilaDivision extends StencilaStyled {
  static styles = sheet.target

  protected renderErrorsContainer(tw: TW) {
    return html`<div
      part="errors"
      class=${tw`border(t slate-200) ${this.hasErrors || 'hidden'}`}
    >
      ${this.renderErrorsSlot(tw)}
    </div>`
  }

  protected renderContentContainer(tw: TW) {
    return html`<div
      part="content"
      class=${tw`border(t slate-200) p-2 ${this.isExpanded || 'hidden'}`}
    >
      ${this.renderContentSlot(tw)}
    </div>`
  }

  render() {
    const mode = currentMode()
    return mode < Mode.Inspect
      ? html`${this.renderCssSlot(tw)} ${this.renderContentSlot(tw)}`
      : html`<div
          part="base"
          class=${tw`my-4 rounded border(& slate-200) overflow-hidden`}
        >
          <div
            part="header"
            class=${tw`flex justify-between items-center bg-slate-100 p-1 font(mono bold) text(sm slate-800)`}
          >
            <span class=${tw`flex items-center text-base ml-1 ml-1 p-1`}>
              <stencila-icon name="brush"></stencila-icon>
            </span>
            <span class=${tw`mr-2`}>div</span>
            ${this.renderTextEditor(tw)} ${this.renderLanguageMenu(tw)}
            ${this.renderViewCssButton(tw)} ${this.renderExpandButton(tw)}
          </div>
          ${this.renderErrorsContainer(tw)} ${this.renderCssViewer(tw)}
          ${this.renderContentContainer(tw)}
        </div>`
  }
}
