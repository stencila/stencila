import { html } from 'lit'
import { customElement } from 'lit/decorators'
import { TW } from 'twind'
import { currentMode, isContentWriteable, Mode } from '../../mode'
import { twSheet } from '../utils/css'
import StencilaStyled from './styled'

const { tw, sheet } = twSheet()

/**
 * A component representing a Stencila `Span` node
 */
@customElement('stencila-span')
export default class StencilaSpan extends StencilaStyled {
  static styles = sheet.target

  static color = 'blue'

  static formats = ['markdown', 'yaml', 'json']

  protected renderErrorsContainer(tw: TW) {
    return html`<span
      part="errors"
      class=${this.hasErrors
        ? tw`bg-red-50 border(l ${StencilaSpan.color}-200) pt-1`
        : tw`hidden`}
    >
      ${this.renderErrorsSlot(tw)}
    </span>`
  }

  protected renderContentContainer(tw: TW) {
    // prettier-ignore
    return html`<span
      part="content"
      class=${this.isExpanded
        ? tw`inline-flex border(l ${StencilaSpan.color}-200) py-1 px-2 ${
            isContentWriteable() ? 'whitespace-pre' : ''
          }`
        : tw`hidden`}
    >${this.renderContentSlot(tw)}</span>`
  }

  render() {
    const mode = currentMode()

    const toggleSelected = () => this.toggleSelected()

    return mode < Mode.Inspect
      ? html`${this.renderCssSlot(tw)} ${this.renderContentSlot(tw)}`
      : html`<span
          part="base"
          contenteditable="false"
          class=${tw`inline-flex my-1 rounded overflow-hidden whitespace-normal border(& ${
            StencilaSpan.color
          }-200) ${this.selected ? `ring-1` : ''}`}
        >
          <span
            part="start"
            class=${tw`inline-flex items-center bg-${StencilaSpan.color}-50
                       py-0.5 px-1
                       font(mono bold) text(sm ${StencilaSpan.color}-700)`}
            @mousedown=${toggleSelected}
          >
            <span class=${tw`inline-flex items-center text-base ml-1`}>
              <stencila-icon name="brush"></stencila-icon>
            </span>
            <span class=${tw`ml-1 mr-2`}>span</span>
            ${this.renderTextEditor(tw)} ${this.renderLanguageMenu(tw)}
          </span>

          ${this.renderErrorsContainer(tw)} ${this.renderCssSlot(tw)}
          ${this.renderContentContainer(tw)}

          <span
            part="end"
            contenteditable="false"
            class=${tw`inline-flex items-center bg-${StencilaSpan.color}-50
                      border(l ${StencilaSpan.color}-200) px-1 text(sm ${StencilaSpan.color}-700)`}
            @mousedown=${toggleSelected}
          >
            ${this.renderDownloadButton(
              StencilaSpan.formats,
              StencilaSpan.color
            )}
          </span>
        </span>`
  }
}
