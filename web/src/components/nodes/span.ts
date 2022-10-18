import { html } from 'lit'
import { customElement } from 'lit/decorators'
import { TW } from 'twind'
import { currentMode, Mode } from '../../mode'
import { twSheet } from '../utils/css'
import StencilaStyled from './styled'

const { tw, sheet } = twSheet()

/**
 * A component representing a Stencila `Span` node
 */
@customElement('stencila-span')
export default class StencilaSpan extends StencilaStyled {
  static styles = sheet.target

  protected renderErrorsContainer(tw: TW) {
    return html`<span
      part="errors"
      class=${tw`inline-flex bg-red-50 border(l slate-200) ${
        this.hasErrors || 'hidden'
      }`}
    >
      ${this.renderErrorsSlot(tw)}
    </span>`
  }

  protected renderContentContainer(tw: TW) {
    return html`<span
      part="content"
      class=${tw`inline-flex border(l slate-200) py-1 px-2 ${
        this.isExpanded || 'hidden'
      }`}
    >
      ${this.renderContentSlot(tw)}
    </span>`
  }

  render() {
    const mode = currentMode()
    return mode < Mode.Inspect
      ? html`${this.renderCssSlot(tw)} ${this.renderContentSlot(tw)}`
      : html`<span
          part="base"
          class=${tw`inline-flex rounded overflow-hidden border(& slate-200)`}
        >
          <span
            part="start"
            class=${tw`inline-flex items-center bg-slate-100 py-0.5 px-1 font(mono bold) text(sm slate-800)`}
          >
            <span class=${tw`text-base ml-1`}>
              <stencila-icon name="brush"></stencila-icon>
            </span>
            <span class=${tw`ml-1 mr-2 `}>span</span>
            ${this.renderTextEditor(tw)} ${this.renderLanguageMenu(tw)}
          </span>
          ${this.renderErrorsContainer(tw)} ${this.renderCssSlot(tw)}
          ${this.renderContentContainer(tw)}
        </span>`
  }
}
