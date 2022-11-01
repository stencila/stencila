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

  static color = 'blue'

  static formats = ['markdown', 'yaml', 'json']

  protected renderErrorsContainer(tw: TW) {
    return html`<span
      part="errors"
      class=${tw`bg-red-50 border(l ${StencilaSpan.color}-200) pt-1 ${
        this.hasErrors || 'hidden'
      }`}
    >
      ${this.renderErrorsSlot(tw)}
    </span>`
  }

  protected renderContentContainer(tw: TW) {
    return html`<span
      part="content"
      class=${tw`inline-flex border(l ${StencilaSpan.color}-200) py-1 px-2 ${
        this.isExpanded || 'hidden'
      }`}
    >
      ${this.renderContentSlot(tw)}
    </span>`
  }

  protected renderContentSlot(tw: TW) {
    return this.isReadOnly()
      ? html`<slot
          name="content"
          @slotchange=${(event: Event) => this.onContentSlotChange(event)}
        ></slot>`
      : html`<stencila-prose-editor
          inline-only
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
      : html`<span
          part="base"
          class=${tw`inline-flex my-1 rounded overflow-hidden border(& ${StencilaSpan.color}-200)`}
        >
          <span
            part="start"
            class=${tw`inline-flex items-center bg-${StencilaSpan.color}-50 py-0.5 px-1
                       font(mono bold) text(sm ${StencilaSpan.color}-700)`}
          >
            <span class=${tw`text-base ml-1`}>
              <stencila-icon name="brush"></stencila-icon>
            </span>
            <span class=${tw`ml-1 mr-2 `}>span</span>
            ${this.renderTextEditor(tw)} ${this.renderLanguageMenu(tw)}
          </span>

          ${this.renderErrorsContainer(tw)} ${this.renderCssSlot(tw)}
          ${this.renderContentContainer(tw)}

          <span
            part="end"
            class=${tw`inline-flex items-center bg-${StencilaSpan.color}-50
                      border(l ${StencilaSpan.color}-200) px-1 text(sm ${StencilaSpan.color}-700)`}
          >
            ${this.renderEntityDownload(
              StencilaSpan.formats,
              StencilaSpan.color
            )}
          </span>
        </span>`
  }
}
