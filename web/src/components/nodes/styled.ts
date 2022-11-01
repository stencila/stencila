import { html } from 'lit'
import { state } from 'lit/decorators'
import { TW } from 'twind'
import { currentMode, Mode } from '../../mode'
import StencilaCodeExecutable from './code-executable'

/**
 * A base class for Stencila `Styled` nodes `Division` and `Span`
 */
export default class StencilaStyled extends StencilaCodeExecutable {
  static color = 'blue'

  /**
   * Whether the `content` is visible
   */
  @state()
  protected isExpanded = true

  /**
   * Whether the generated CSS is visible
   */
  @state()
  private isCssVisible = false

  /**
   * The CSS class name of the `content`
   *
   * Always added to the content element but only needed if there is a change to
   * the `css` slot at which time a stylesheet will be constructed that uses this class.
   */
  protected cssClass = `st-${Math.floor(Math.random() * 1e9)}`

  /**
   * The CSS rules as a raw CSS string
   *
   * This is captured from the document's global stylesheet on load so that
   * if needs be it can be passed on to the <stencila-prose-editor>. It is
   * also updated when the CSS slot changes so that it can be passed
   * through to the editor again. To trigger a rerender it is a @state.
   */
  @state()
  protected cssRules: string

  /**
   * The CSS stylesheet that is constructed for the `content` if the
   * CSS changes.
   */
  protected cssStyleSheet?: CSSStyleSheet

  /**
   * An observer to update `cssStyleSheet` when the content of the `css`
   * slot changes
   */
  private cssObserver?: MutationObserver

  /**
   * Handle a change to the `css` slot to initialize `cssObserver`
   */
  private onCssSlotChange(event: Event) {
    const cssElem = (event.target as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0]

    // Handle initial load of slot
    this.onCssChanged(cssElem.textContent ?? '', true)

    this.cssObserver = new MutationObserver(() => {
      // Handle subsequent mutations
      this.onCssChanged(cssElem.textContent ?? '')
    })
    this.cssObserver.observe(cssElem, {
      subtree: true,
      characterData: true,
    })
  }

  /**
   * Handle a change to the transpiled CSS
   *
   * Updates the custom stylesheet for this `Styled` creating a new
   * `CSSStyleSheet` if necessary.
   */
  private onCssChanged(css: string, initial = false) {
    // If necessary create a new stylesheet for the new CSS
    if (this.cssStyleSheet === undefined) {
      this.cssStyleSheet = new CSSStyleSheet()
      document.adoptedStyleSheets = [
        ...document.adoptedStyleSheets,
        this.cssStyleSheet,
      ]
    }

    // Replace the content of the stylesheet with the new CSS
    // Use the unique class name for the element
    let stylesheet = css.replace(':root', `.${this.cssClass}`)
    // Add transitions for all properties if this is not the initial render and the
    // CSS does not have any transitions defined.
    if (!initial && !stylesheet.includes('transition-property:')) {
      stylesheet += `\n\n.${this.cssClass} {
  transition-property: all;
  transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
  transition-duration: 500ms;
}`
    }

    this.cssRules = stylesheet
    this.cssStyleSheet.replaceSync(stylesheet)
  }

  /**
   * Handle a change to the `content` slot to add `cssClass` to it
   */
  protected onContentSlotChange(event: Event) {
    const contentElem = (event.target as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0] as HTMLDivElement

    // Replaces any existing Stencila class (i.e the one generated when HTML was generated)
    const oldClass = [...contentElem.classList].filter((className) =>
      className.startsWith('st-')
    )[0]
    if (oldClass !== undefined) {
      contentElem.classList.replace(oldClass, this.cssClass)
    } else {
      contentElem.classList.add(this.cssClass)
    }
  }

  protected renderTextEditor(tw: TW) {
    const readOnly = this.isReadOnly()
    return html`<stencila-code-editor
      class=${tw`min-w-0 w-full rounded overflow-hidden 
                 border(& ${StencilaStyled.color}-200) focus:border(& ${StencilaStyled.color}-400)
                 focus:ring(2 ${StencilaStyled.color}-100) bg-${StencilaStyled.color}-50 font-normal`}
      language=${this.programmingLanguage}
      single-line
      line-wrapping
      no-controls
      placeholder="Not yet compiled or no rules"
      ?read-only=${readOnly}
      ?disabled=${readOnly}
      @stencila-ctrl-enter=${() => this.execute()}
    >
      <slot name="text" slot="code"></slot>
    </stencila-code-editor>`
  }

  protected renderErrorsSlot(tw: TW) {
    return html`<slot
      name="errors"
      @slotchange=${(event: Event) => this.onErrorsSlotChange(event)}
    ></slot>`
  }

  protected renderViewCssButton(tw: TW) {
    return html` <stencila-icon-button
      name=${this.isCssVisible ? 'eye-slash' : 'eye'}
      color=${StencilaStyled.color}
      adjust="ml-2"
      @click=${() => {
        this.isCssVisible = !this.isCssVisible
      }}
      @keydown=${(event: KeyboardEvent) => {
        if (event.key == 'Enter') {
          event.preventDefault()
          this.isCssVisible = !this.isCssVisible
        }
      }}
    >
    </stencila-icon-button>`
  }

  protected renderCssSlot(tw: TW) {
    return html` <slot
      class=${tw`hidden`}
      name="css"
      @slotchange=${(event: Event) => this.onCssSlotChange(event)}
    ></slot>`
  }

  protected renderCssViewer(tw: TW) {
    return html`<div class=${this.isCssVisible ? 'block' : 'hidden'}>
      <div
        part="css-header"
        class=${tw`flex justify-between items-center bg-${StencilaStyled.color}-50 border(t b ${StencilaStyled.color}-200)
                   p-1 font(sans) text(xs ${StencilaStyled.color}-700)`}
      >
        <span class=${tw`flex items-center`}>
          <stencila-icon
            name="css-color"
            class=${tw`ml-1 mr-1 text-base`}
          ></stencila-icon>
          <span>CSS</span>
        </span>
      </div>

      <stencila-code-editor part="css" language="css" read-only no-controls>
        <slot
          name="css"
          slot="code"
          @slotchange=${(event: Event) => this.onCssSlotChange(event)}
        ></slot>
      </stencila-code-editor>
    </div>`
  }

  protected renderExpandButton(
    tw: TW,
    direction: 'vertical' | 'horizontal' = 'vertical'
  ) {
    return html`<stencila-icon-button
      name=${direction === 'vertical' ? 'chevron-right' : 'chevron-left'}
      color=${StencilaStyled.color}
      adjust=${`ml-2 rotate-${
        this.isExpanded ? (direction === 'vertical' ? 90 : 180) : 0
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
}
