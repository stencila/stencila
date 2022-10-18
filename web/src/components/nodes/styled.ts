import { html } from 'lit'
import { state } from 'lit/decorators'
import { TW } from 'twind'
import { currentMode, Mode } from '../../mode'
import StencilaCodeExecutable from './code-executable'

/**
 * A base class for Stencila `Styled` nodes `Division` and `Span`
 */
export default class StencilaStyled extends StencilaCodeExecutable {
  /**
   * Whether the `content` is visible
   */
  @state()
  protected isExpanded: boolean = true

  /**
   * Whether the generated CSS is visible
   */
  @state()
  private isCssVisible: boolean = false

  /**
   * The CSS class name of the `content`
   *
   * Always added to the content element but only needed if there is a change to
   * the `css` slot at which time a stylesheet will be constructed that uses this class.
   */
  private cssClass: string = `st-${Math.floor(Math.random() * 1e9)}`

  /**
   * Handle a change to the `content` slot to add `cssClass` to it
   */
  private onContentSlotChange(event: Event) {
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

  /**
   * The CSS stylesheet that is constructed for the `content` if the
   * CSS changes
   */
  private cssStyleSheet?: CSSStyleSheet

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
  private onCssChanged(css: string, initial: boolean = false) {
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
      stylesheet += `.${this.cssClass} {
  transition-property: all;
  transition-timing-function: cubic-bezier(0.4, 0, 0.2, 1);
  transition-duration: 500ms;
}`
    }
    this.cssStyleSheet.replaceSync(stylesheet)
  }

  protected renderTextEditor(tw: TW) {
    return html`<stencila-code-editor
      class=${tw`min-w-0 w-full rounded overflow-hidden border(& slate-200) focus:border(& slate-400) focus:ring(2 slate-100) bg-slate-50 font-normal pr-1`}
      language=${this.programmingLanguage}
      ?read-only=${currentMode() < Mode.Alter}
      single-line
      line-wrapping
      no-controls
      @stencila-ctrl-enter=${() => this.execute()}
    >
      <code slot="code">${this.text}</code>
    </stencila-code-editor>`
  }

  protected renderLanguageMenu(tw: TW) {
    return html`<stencila-executable-language
      class=${tw`ml-2 text(base slate-500)`}
      programming-language=${this.programmingLanguage}
      guess-language=${this.guessLanguage == 'true'}
      color="slate"
    ></stencila-executable-language>`
  }

  protected renderErrorsSlot(tw: TW) {
    return html` <slot
      name="errors"
      @slotchange=${(event: Event) => this.onErrorsSlotChange(event)}
    ></slot>`
  }

  protected renderViewCssButton(tw: TW) {
    return html` <stencila-icon-button
      name=${this.isCssVisible ? 'eye-slash' : 'eye'}
      color="slate"
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
        class=${tw`flex justify-between items-center bg-slate-100 border(t b slate-200) p-1 font(mono bold) text(sm slate-800)`}
      >
        <span class=${tw`flex items-center`}>
          <stencila-icon
            name="code"
            class=${tw`ml-2 mr-3 text-base`}
          ></stencila-icon>
          <span>css</span>
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
      color="slate"
      adjust=${`ml-2 rotate-${
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

  protected renderContentSlot(tw: TW) {
    return html` <slot
      name="content"
      @slotchange=${(event: Event) => this.onContentSlotChange(event)}
    ></slot>`
  }
}
