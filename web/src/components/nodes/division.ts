import { html } from 'lit'
import { customElement } from 'lit/decorators'
import { TW } from 'twind'
import { currentMode, isContentWriteable, Mode } from '../../mode'
import { twSheet } from '../utils/css'
import StencilaStyled from './styled'

const { tw, sheet } = twSheet()

/**
 * A component representing a Stencila `Division` node
 */
@customElement('stencila-division')
export default class StencilaDivision extends StencilaStyled {
  static styles = sheet.target

  static color = 'blue'

  static formats = ['markdown', 'python', 'javascript', 'r', 'yaml', 'json']

  protected renderErrorsContainer(tw: TW) {
    return html`<div
      part="errors"
      class=${this.hasErrors
        ? tw`border(t ${StencilaDivision.color}-200)`
        : tw`hidden`}
    >
      ${this.renderErrorsSlot(tw)}
    </div>`
  }

  protected renderContentContainer(tw: TW) {
    // prettier-ignore
    return html`<div
      part="content"
      class=${this.isExpanded
        ? tw`border(t ${StencilaDivision.color}-200) p-2 ${
            isContentWriteable() ? 'whitespace-pre' : ''
          }`
        : tw`hidden`}
    >${this.renderContentSlot(tw)}</div>`
  }

  render() {
    const mode = currentMode()

    const toggleSelected = () => this.toggleSelected()

    return mode < Mode.Inspect
      ? html`${this.renderCssSlot(tw)} ${this.renderContentSlot(tw)}`
      : html`<div
          part="base"
          class=${tw`my-4 rounded overflow-hidden whitespace-normal border(& ${
            StencilaDivision.color
          }-200) ${this.selected ? `ring-1` : ''}`}
        >
          <div
            part="header"
            contenteditable="false"
            class=${tw`flex justify-between items-center bg-${StencilaDivision.color}-50
                       p-1 font(mono bold) text(sm ${StencilaDivision.color}-700)`}
            @mousedown=${toggleSelected}
          >
            <span class=${tw`flex items-center text-base ml-1 mr-2`}>
              <stencila-icon name="brush"></stencila-icon>
            </span>
            <span class=${tw`mr-2`}>div</span>
            ${this.renderTextEditor(tw)} ${this.renderLanguageMenu(tw)}
            ${this.renderViewCssButton(tw)} ${this.renderExpandButton(tw)}
          </div>

          ${this.renderErrorsContainer(tw)} ${this.renderCssViewer(tw)}
          ${this.renderContentContainer(tw)}

          <div
            part="footer"
            contenteditable="false"
            class=${tw`grid justify-items-end items-center bg-${StencilaDivision.color}-50
                       border(t ${StencilaDivision.color}-200) p-1 text(sm ${StencilaDivision.color}-700)`}
            @mousedown=${toggleSelected}
          >
            ${this.renderDownloadButton(
              StencilaDivision.formats,
              StencilaDivision.color
            )}
          </div>
        </div>`
  }
}
