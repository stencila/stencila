import '@shoelace-style/shoelace/dist/components/dropdown/dropdown'
import '@shoelace-style/shoelace/dist/components/menu/menu'
import { css, html } from 'lit'
import { customElement, property, state } from 'lit/decorators'
import { TW } from 'twind'
import { isCodeWriteable } from '../../mode'
import '../base/icon'
import '../base/icon-button'
import { twSheet } from '../utils/css'
import StencilaElement from '../utils/element'
import StencilaExecutable from './executable'

/**
 * A base class for Stencila `Math` nodes `MathBlock` and `MathFragment`
 *
 * Although in the Stencila Schema `Math` does not extend `Executable`, we use it
 * as a base path here is inherit `isReadOnly`, `compile` and other methods.
 *
 * @slot text The `Math.text` property
 * @slot errors The `Math.errors` property
 * @slot mathml The `Math.mathml` property
 */
export default class StencilaMath extends StencilaExecutable {
  /**
   * The `Math.mathLanguage` property
   */
  @property({ attribute: 'math-language' })
  mathLanguage = ''

  /**
   * Whether the `mathml` is visible
   */
  @state()
  protected isExpanded = true

  /**
   * An observer to update the display MathML when the raw `mathml` slot changes
   */
  private mathmlObserver: MutationObserver

  /**
   * Handle a change, including on initial load, of the `mathml` slot
   */
  protected onMathMLSlotChange(event: Event) {
    const mathmlElem = (event.target as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0]

    this.onMathMLChange(mathmlElem.textContent ?? '')

    this.mathmlObserver = new MutationObserver(() => {
      this.onMathMLChange(mathmlElem.textContent ?? '')
    })
    this.mathmlObserver.observe(mathmlElem, {
      subtree: true,
      characterData: true,
    })
  }

  /**
   * When there are changes in the MathML set the HTML of the display element
   */
  protected onMathMLChange(mathml: string) {
    const display = this.renderRoot.querySelector('[data-display]')!
    if (display) display.innerHTML = mathml
  }

  protected renderTextEditor(tw: TW, color: string) {
    const readOnly = !isCodeWriteable()

    return html`<stencila-code-editor
      class=${tw`min-w-0 w-full rounded overflow-hidden border(& ${color}-200) bg-${color}-50
                 focus:border(& ${color}-400) focus:ring(2 ${color}-100) font-normal`}
      language=${this.mathLanguage}
      single-line
      line-wrapping
      no-controls
      ?read-only=${readOnly}
      ?disabled=${readOnly}
      @stencila-ctrl-enter=${() => this.compile()}
    >
      <slot name="text" slot="code"></slot>
    </stencila-code-editor>`
  }

  protected renderLanguageMenu(tw: TW, color: string) {
    const readOnly = !isCodeWriteable()

    return html`<stencila-math-language
      class=${tw`ml-2 text(base ${color}-500)`}
      math-language=${this.mathLanguage}
      ?disabled=${readOnly}
    ></stencila-math-language>`
  }

  protected renderExpandButton(
    tw: TW,
    color: string,
    direction: 'vertical' | 'horizontal' = 'vertical'
  ) {
    return html`<stencila-icon-button
      name=${direction === 'vertical' ? 'chevron-right' : 'chevron-left'}
      color=${color}
      adjust=${`ml-1 rotate-${
        this.isExpanded ? (direction === 'vertical' ? '90' : 180) : '0'
      } transition-transform`}
      @click=${() => {
        this.isExpanded = !this.isExpanded
      }}
      @keydown=${(event: KeyboardEvent) => {
        if (
          event.key == 'Enter' ||
          (event.key == 'ArrowUp' && this.isExpanded) ||
          (event.key == 'ArrowDown' && !this.isExpanded)
        ) {
          event.preventDefault()
          this.isExpanded = !this.isExpanded
        }
      }}
    >
    </stencila-icon-button>`
  }

  protected renderErrorsSlot(tw: TW) {
    return html`<slot name="errors"></slot>`
  }

  protected renderMathMLSlot(tw: TW, inline: boolean, cls = '') {
    return html`
      <slot
        name="mathml"
        class=${tw`hidden`}
        @slotchange=${(event: Event) => this.onMathMLSlotChange(event)}
      ></slot>

      ${inline
        ? html`<span data-display class=${tw(cls)}></span>`
        : html`<div data-display class=${tw(cls)}></div>`}
    `
  }
}

const { tw, sheet } = twSheet()

/**
 * A component for changing the `mathLanguage` property of a `Math` node
 */
@customElement('stencila-math-language')
export class StencilaMathLanguage extends StencilaElement {
  static styles = [
    sheet.target,
    css`
      sl-menu-item::part(label) {
        line-height: 1;
      }
    `,
  ]

  static languages = [
    ['asciimath', 'AsciiMath'],
    ['mathml', 'MathML'],
    ['tex', 'TeX'],
  ]

  /**
   * The `Math.mathLanguage` property
   */
  @property({ attribute: 'math-language', reflect: true })
  mathLanguage: string

  /**
   * Whether the menu is disabled
   */
  @property({ type: Boolean })
  disabled = false

  render() {
    const languages = StencilaMathLanguage.languages
    const language = this.mathLanguage.toLowerCase()

    let icon = 'code'
    for (const [value, title, ...aliases] of languages) {
      if (language === value || aliases.includes(language)) {
        icon = value
        break
      }
    }

    return html`
      <sl-dropdown class=${tw`flex items-center`} ?disabled=${this.disabled}>
        <stencila-icon-button
          slot="trigger"
          name=${icon}
          color="blue"
          ?disabled=${this.disabled}
        >
        </stencila-icon-button>

        <sl-menu
          @sl-select="${(event: CustomEvent) => {
            const value = event.detail.item.value
            if (this.mathLanguage !== value) {
              this.changeProperty('mathLanguage', value)
            }
          }}}"
        >
          ${languages.map(
            ([value, title, ...aliases]) =>
              html`<sl-menu-item
                value=${value}
                ?checked=${language == value || aliases.includes(language)}
              >
                <stencila-icon
                  slot="prefix"
                  name="${value}-color"
                ></stencila-icon>
                <span class=${tw`text-sm`}>${title}</span>
              </sl-menu-item>`
          )}
        </sl-menu>
      </sl-dropdown>
    `
  }
}
