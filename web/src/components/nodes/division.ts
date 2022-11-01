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

  static color = 'blue'

  static formats = ['markdown', 'python', 'javascript', 'r', 'yaml', 'json']

  protected renderErrorsContainer(tw: TW) {
    return html`<div
      part="errors"
      class=${tw`border(t ${StencilaDivision.color}-200) ${
        this.hasErrors || 'hidden'
      }`}
    >
      ${this.renderErrorsSlot(tw)}
    </div>`
  }

  protected renderContentContainer(tw: TW) {
    return html`<div
      part="content"
      class=${tw`border(t ${StencilaDivision.color}-200) p-2 ${
        this.isExpanded || 'hidden'
      }`}
    >
      ${this.renderContentSlot(tw)}
    </div>`
  }

  protected renderContentSlot(tw: TW) {
    return this.isReadOnly()
      ? html`<slot
          name="content"
          @slotchange=${(event: Event) => this.onContentSlotChange(event)}
        ></slot>`
      : html`<stencila-prose-editor
          css-class=${this.cssClass}
          css-rules=${this.cssRules}
          ><slot
            name="content"
            slot="content"
            class=${tw`hidden`}
            @slotchange=${(event: Event) => this.onContentSlotChange(event)}
          ></slot
        ></stencila-prose-editor>`
  }

  render() {
    const mode = currentMode()
    return mode < Mode.Inspect
      ? html`${this.renderCssSlot(tw)} ${this.renderContentSlot(tw)}`
      : html`<div
          part="base"
          class=${tw`my-4 rounded border(& ${StencilaDivision.color}-200) overflow-hidden`}
        >
          <div
            part="header"
            class=${tw`flex justify-between items-center bg-${StencilaDivision.color}-50
                       p-1 font(mono bold) text(sm ${StencilaDivision.color}-700)`}
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
            class=${tw`grid justify-items-end items-center bg-${StencilaDivision.color}-50
                       border(t ${StencilaDivision.color}-200) p-1 text(sm ${StencilaDivision.color}-700)`}
          >
            ${this.renderEntityDownload(
              StencilaDivision.formats,
              StencilaDivision.color
            )}
          </div>
        </div>`
  }
}
