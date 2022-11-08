import { SlInput } from '@shoelace-style/shoelace'
import { css, html, PropertyValueMap } from 'lit'
import { customElement, property, state } from 'lit/decorators'
import { isCodeWriteable, isContentWriteable } from '../../mode'

import { twSheet } from '../utils/css'
import StencilaCodeExecutable, {
  StencilaExecutableLanguage,
} from './code-executable'
import './for-iteration'

const { tw, sheet } = twSheet()

/**
 * A component representing a Stencila `For` document node
 */
@customElement('stencila-for')
export default class StencilaFor extends StencilaCodeExecutable {
  static styles = [
    sheet.target,
    css`
      sl-input::part(base) {
        font-family: monospace;
      }
    `,
  ]

  static color = 'blue'

  static formats = ['markdown', 'yaml', 'json']

  /**
   * The `For.symbol` property
   */
  @property({ reflect: true })
  symbol: string

  /**
   * Whether the `content` is visible
   */
  @state()
  private isContentExpanded = false

  /**
   * Whether or not the `content` slot has content
   */
  @state()
  private hasContent = false

  /**
   * An observer to update `hasContent`
   */
  private contentObserver: MutationObserver

  /**
   * Handle a change to the `content` slot
   *
   * Initializes `hasContent` and a `MutationObserver` to watch for changes
   * to the number of elements in the slot and change `hasContent` accordingly.
   */
  private onContentSlotChange(event: Event) {
    const contentElem = (event.target as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0] as HTMLElement | undefined

    if (contentElem) {
      this.hasContent = contentElem.childElementCount > 0

      this.contentObserver = new MutationObserver(() => {
        this.hasContent = contentElem.childElementCount > 0
      })
      this.contentObserver.observe(contentElem, {
        childList: true,
      })
    }
  }

  /**
   * Whether the `otherwise` content is visible
   */
  @state()
  private isOtherwiseExpanded = false

  /**
   * Whether or not the `otherwise` slot has content
   */
  @state()
  private hasOtherwise = false

  /**
   * An observer to update `hasOtherwise`
   */
  private otherwiseObserver: MutationObserver

  /**
   * Handle a change to the `otherwise` slot
   *
   * Initializes `hasOtherwise` as well as a `MutationObserver` to watch for changes
   * to the number of elements in the slot and change `hasOtherwise` accordingly.
   */
  private onOtherwiseSlotChange(event: Event) {
    const otherwiseElem = (event.target as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0] as HTMLElement | undefined

    if (otherwiseElem) {
      this.hasOtherwise = otherwiseElem.childElementCount > 0

      this.otherwiseObserver = new MutationObserver(() => {
        this.hasOtherwise = otherwiseElem.childElementCount > 0
      })
      this.otherwiseObserver.observe(otherwiseElem, {
        childList: true,
      })
    }
  }

  /**
   * Whether the `iterations` items are visible
   */
  @state()
  private isIterationsExpanded = false

  /**
   * Whether or not the for block has any `iterations`
   */
  @state()
  private hasIterations = false

  /**
   * An observer to update `isIterationsEmpty`
   */
  private iterationsObserver: MutationObserver

  /**
   * Handle a change to the `iterations` slot
   *
   * Initializes `hasIterations` as well as a `MutationObserver` to watch for changes
   * to the number of elements in the slot and change `hasIterations` accordingly.
   */
  private onIterationsSlotChange(event: Event) {
    const iterationsElem = (event.target as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0]

    this.hasIterations = iterationsElem.childElementCount > 0

    this.iterationsObserver = new MutationObserver(() => {
      this.hasIterations = iterationsElem.childElementCount > 0
    })
    this.iterationsObserver.observe(iterationsElem, {
      childList: true,
    })
  }

  /**
   * Override to set `isOtherwiseExpanded` based on the changes in `hasIterations` and `hasOtherwise`.
   * This allows expansion/contraction based on changes to which clause is active as well as based on
   * user interaction.
   */
  protected update(
    changedProperties: PropertyValueMap<any> | Map<PropertyKey, unknown>
  ): void {
    super.update(changedProperties)

    if (
      changedProperties.has('hasIterations') ||
      changedProperties.has('hasOtherwise')
    ) {
      this.isOtherwiseExpanded = !this.hasIterations && this.hasOtherwise
    }

    if (changedProperties.has('hasIterations')) {
      this.isIterationsExpanded = this.hasIterations
    }
  }

  protected renderSymbolInput() {
    const readOnly = !isCodeWriteable()

    const replace = (event: Event): boolean => {
      const input = event.target as SlInput
      if (input.reportValidity()) {
        const symbol = (event.target as HTMLInputElement).value
        this.changeProperty('symbol', symbol)
        return true
      }
      return false
    }

    // TODO: Use <stencila-input> here for consistent validation popups
    // Do not use `required` on <sl-input> to avoid browser validation popups
    // and instead use <stencila-input>
    return html`<sl-input
      class=${tw`min-w-0 w-1/4`}
      size="small"
      placeholder="item"
      pattern="[a-zA-Z][a-zA-Z0-0_]*"
      value=${this.symbol}
      ?disabled=${readOnly}
      @focus=${() => this.deselect()}
      @mousedown=${(event: Event) => {
        this.deselect()
        event.stopPropagation()
      }}
      @sl-change=${replace}
      @sl-blur=${replace}
      @keypress=${(event: KeyboardEvent) => {
        // Execute the `For` on `Ctrl+Enter`
        if (event.key == 'Enter' && event.ctrlKey) {
          event.preventDefault()
          if (replace(event)) {
            this.execute('Single')
          }
        }
      }}
    ></sl-input>`
  }

  protected renderTextEditor() {
    const readOnly = !isCodeWriteable()

    return html`<stencila-code-editor
      class=${tw`min-w-0 w-full rounded overflow-hidden border(& ${StencilaFor.color}-200)
                focus:border(& ${StencilaFor.color}-400) focus:ring(2 ${StencilaFor.color}-100) bg-${StencilaFor.color}-50 font-normal`}
      language=${this.programmingLanguage}
      single-line
      line-wrapping
      no-controls
      placeholder="items"
      ?read-only=${readOnly}
      ?disabled=${readOnly}
      @focus=${() => this.deselect()}
      @mousedown=${(event) => {
        this.deselect()
        event.stopPropagation()
      }}
      @stencila-ctrl-enter=${() => this.execute()}
    >
      <slot name="text" slot="code"></slot>
    </stencila-code-editor>`
  }

  protected renderContentContainer() {
    return html`<div
      part="content"
      class=${tw`border(t ${StencilaFor.color}-200) p-2 ${
        this.isContentExpanded || 'hidden'
      }`}
    >
      ${!this.hasContent
        ? html`<p class=${tw`text(center gray-300)`}>No content</p>`
        : ''}
      <slot
        name="content"
        @slotchange=${(event: Event) => this.onContentSlotChange(event)}
      ></slot>
    </div>`
  }

  protected renderOtherwiseContainer() {
    return html`<div
      part="otherwise"
      class=${tw`border(t ${StencilaFor.color}-200) p-2 ${
        this.isOtherwiseExpanded || 'hidden'
      }`}
    >
      ${!this.hasOtherwise
        ? html`<p class=${tw`text(center gray-300)`}>No content</p>`
        : ''}
      <slot
        name="otherwise"
        @slotchange=${(event: Event) => this.onOtherwiseSlotChange(event)}
      ></slot>
    </div>`
  }

  protected render() {
    const readOnly = !isCodeWriteable()

    const toggleSelected = () => this.toggleSelected()

    const programmingLanguageMenu = html`<stencila-code-language
      class=${tw`ml-2 text(base gray-500)`}
      color=${StencilaFor.color}
      programming-language=${this.programmingLanguage}
      ?guess-language=${this.guessLanguage == 'true'}
      ?is-guessable=${true}
      ?executable-only=${true}
      exclude='["tailwind"]'
      ?disabled=${readOnly}
      @stencila-document-patch=${(event: CustomEvent) => {
        // Update `this.programmingLanguage` (and `guessLanguage` for completeness)
        // so that the code editor language updates
        const elem = event.target as StencilaExecutableLanguage
        this.programmingLanguage = elem.programmingLanguage
        this.guessLanguage = elem.guessLanguage.toString()
      }}
    ></stencila-code-language>`

    const expandButton = (
      property:
        | 'isContentExpanded'
        | 'isOtherwiseExpanded'
        | 'isIterationsExpanded'
    ) => html`<stencila-icon-button
      name="chevron-right"
      color=${StencilaFor.color}
      adjust=${`ml-2 rotate-${
        this[property] ? '90' : '0'
      } transition-transform`}
      @click=${() => {
        this[property] = !this[property]
      }}
      @keydown=${(event: KeyboardEvent) => {
        if (
          event.key == 'Enter' ||
          (event.key == 'ArrowUp' && this[property]) ||
          (event.key == 'ArrowDown' && !this[property])
        ) {
          event.preventDefault()
          this[property] = !this[property]
        }
      }}
    >
    </stencila-icon-button>`

    const errorsContainer = html`<div
      part="errors"
      class=${tw`border(t ${StencilaFor.color}-200) ${
        this.hasErrors || 'hidden'
      }`}
    >
      <slot
        name="errors"
        @slotchange=${(event: Event) => this.onErrorsSlotChange(event)}
      ></slot>
    </div>`

    const contentExpandButton = expandButton('isContentExpanded')

    const otherwiseExpandButton = expandButton('isOtherwiseExpanded')

    const otherwiseHeader = html`<div
      part="otherwise-header"
      class=${tw`flex justify-between items-center bg-${StencilaFor.color}-50 border(t ${StencilaFor.color}-200)
                 p-1 font(mono bold) text(sm ${StencilaFor.color}-700)`}
      @mousedown=${toggleSelected}
    >
      <span class=${tw`flex items-center`}>
        <span
          class=${tw`flex items-center text-base ml-1 mr-2 p-1 ${
            !this.hasIterations && this.hasOtherwise
              ? `rounded-full border(& ${StencilaFor.color}-300) bg-${StencilaFor.color}-100`
              : ''
          }`}
        >
          <stencila-icon name="arrow-return-right"></stencila-icon>
        </span>
        <span>else</span>
      </span>
      ${otherwiseExpandButton}
    </div>`

    const iterationsExpandButton = expandButton('isIterationsExpanded')

    const iterationsHeader = html`<div
      part="iterations-header"
      class=${tw`flex justify-between items-center bg-${
        StencilaFor.color
      }-50 border(t ${StencilaFor.color}-200)
                 p-1 font(mono bold) text(sm ${StencilaFor.color}-700) ${
        this.hasIterations || 'hidden'
      }`}
      @mousedown=${toggleSelected}
    >
      <span class=${tw`flex items-center`}>
        <span
          class=${tw`flex items-center text-base ml-1 mr-2 p-1 ${
            this.hasIterations
              ? `rounded-full border(& ${StencilaFor.color}-300) bg-${StencilaFor.color}-100`
              : ''
          }`}
        >
          <stencila-icon name="list"></stencila-icon>
        </span>
        <span>items</span>
      </span>
      ${iterationsExpandButton}
    </div>`

    const iterationsContainer = html`<div
      part="iterations"
      class=${tw`${
        this.hasIterations || `border(t ${StencilaFor.color}-200) p-2`
      } ${this.isIterationsExpanded || 'hidden'}`}
    >
      ${!this.hasIterations
        ? html`<p class=${tw`text(center gray-300)`}>No items</p>`
        : ''}
      <slot
        name="iterations"
        @slotchange=${(event: Event) => this.onIterationsSlotChange(event)}
      ></slot>
    </div>`

    return html`<div
      part="base"
      class=${tw`my-4 rounded overflow-hidden border(& ${
        StencilaFor.color
      }-200) ${this.selected ? `ring-1` : ''}`}
    >
      <div
        part="header"
        class=${tw`flex items-center bg-${StencilaFor.color}-50 p-1 font(mono bold) text(sm ${StencilaFor.color}-700)`}
        @mousedown=${toggleSelected}
      >
        <span class=${tw`flex items-center text-base ml-1 mr-2 p-1`}>
          <stencila-icon name="repeat"></stencila-icon>
        </span>
        <span class=${tw`mr-1`}>for</span>
        ${this.renderSymbolInput()}
        <span class=${tw`mx-1`}>in</span>
        ${this.renderTextEditor()} ${programmingLanguageMenu}
        ${contentExpandButton}
      </div>

      ${errorsContainer} ${this.renderContentContainer()} ${otherwiseHeader}
      ${this.renderOtherwiseContainer()} ${iterationsHeader}
      ${iterationsContainer}

      <div
        part="footer"
        class=${tw`grid justify-items-end items-center bg-${StencilaFor.color}-50
                       border(t ${StencilaFor.color}-200) p-1 text(sm ${StencilaFor.color}-700)`}
        @mousedown=${toggleSelected}
      >
        ${this.renderDownloadButton(StencilaFor.formats, StencilaFor.color)}
      </div>
    </div>`
  }
}
