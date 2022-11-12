import { html, PropertyValueMap } from 'lit'
import { customElement, property, state } from 'lit/decorators'
import {
  isCodeExecutable,
  isCodeWriteable,
  isContentWriteable,
} from '../../mode'

import { Patch } from '../../types'
import '../base/icon-button'
import StencilaCodeEditor from '../editors/code-editor/code-editor'
import { twSheet } from '../utils/css'
import './code-error'
import StencilaCodeExecutable, {
  StencilaExecutableLanguage,
} from './code-executable'
import StencilaIf from './if'

const { tw, sheet } = twSheet()

/**
 * A component representing a Stencila `IfClause` document node
 */
@customElement('stencila-if-clause')
export default class StencilaIfClause extends StencilaCodeExecutable {
  static styles = sheet.target

  /**
   * The JSON value sent in a patch when creating a new clause
   */
  static json = {
    type: 'IfClause',
    guessLanguage: true,
  }

  /**
   * The HTML fragment added to the DOM when creating a new clause
   */
  static html = `<stencila-if-clause guess-language="true">
    <div data-prop="errors" slot="errors"></div>
  </stencila-if-clause>`

  /**
   * The `IfClause.isActive` property
   *
   * As for `guessLanguage`, needs to be a string.
   */
  @property({ attribute: 'is-active', reflect: true })
  isActive = 'false'

  /**
   * The index of this clause within an `If` node
   *
   * Used for display and to emit patches with address for this clause.
   */
  @state()
  private index: number

  /**
   * Whether this is the first clause in an `If` node
   */
  @state()
  private isFirst: boolean

  /**
   * Whether this is the last clause in an `If` node
   */
  @state()
  private isLast: boolean

  /**
   * Whether this is an else clause
   */
  @state()
  private isElse: boolean

  /**
   * Whether the clause has any content
   */
  @state()
  private hasContent = false

  /**
   * An observer to update `hasContent`
   */
  private contentObserver: MutationObserver

  /**
   * Handle a change, including on initial load, of the content slot
   */
  onContentSlotChange(event: Event) {
    const contentElem = (event.target as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0]

    this.hasContent = contentElem.childElementCount > 0

    this.contentObserver = new MutationObserver(() => {
      this.hasContent = contentElem.childElementCount > 0
    })
    this.contentObserver.observe(contentElem, {
      childList: true,
    })
  }

  /**
   * Get the parent `If` element
   */
  private getIf(): StencilaIf {
    return this.parentElement!.parentElement! as StencilaIf
  }

  /**
   * Get all the clauses in the parent `If` element
   */
  private getIfClauses(): StencilaIfClause[] {
    return [...this.parentElement!.children] as StencilaIfClause[]
  }

  /**
   * Request an update of all clauses in the parent `If` node
   */
  private requestUpdateAll() {
    this.getIfClauses().forEach((clause: StencilaIfClause) =>
      clause.requestUpdate()
    )
  }

  /**
   * Override of `Element.emitPatch` to make the parent `If` node the `target` of
   * the patch (by using the id of the containing <stencila-if>) and prepending the address
   * with the relative address of this `IfClause`
   */
  protected async emitPatch(patch: Patch) {
    const index = this.getIfClauses().indexOf(this)

    const ops = patch.ops.map((op) => {
      if (op.type === 'Move') {
        return {
          ...op,
          from: ['clauses', ...op.from],
          to: ['clauses', ...op.to],
        }
      } else {
        return {
          ...op,
          address: ['clauses', index, ...op.address],
        }
      }
    })

    return super.emitPatch({
      target: this.getIf().id,
      ops,
    })
  }

  /**
   * Override of `Executable.execute` to execute the parent `If` node by using
   * the id of the containing <stencila-if> element
   */
  protected execute() {
    this.emit('stencila-document-execute', {
      nodeId: this.getIf().id,
      ordering: 'Single',
    })
  }

  /**
   * Override to set `isExpanded` based on the changes in `isActive`. This allows expansion/contraction
   * based on changes to which clause is active as well as based on user interaction
   */
  protected update(
    changedProperties: PropertyValueMap<any> | Map<PropertyKey, unknown>
  ): void {
    super.update(changedProperties)

    const clauses = this.getIfClauses()
    const index = clauses.indexOf(this)
    this.index = index
    this.isFirst = index == 0
    this.isLast = index == clauses.length - 1
    this.isElse = this.isLast && this.text?.trim().length == 0

    if (changedProperties.has('isActive')) {
      this.isExpanded = this.isActive == 'true'
    }
  }

  protected renderContentContainer() {
    const readOnly = !isContentWriteable()
    return html`<div
      part="content"
      class=${tw`border(t ${StencilaIf.color}-200) p-2 ${
        this.isExpanded || 'hidden'
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

  protected render() {
    const label = this.index == 0 ? 'if' : this.isElse ? 'else' : 'elif'
    const iconName =
      label == 'if' || label == 'elif' ? 'arrow-right' : 'arrow-return-right'
    const isActive = this.isActive == 'true'

    const readOnly = !isCodeWriteable()

    // Toggle selection of the parent `If` node
    const toggleSelected = () => {
      const parent = this.getIf()
      parent.toggleSelected()
    }

    // Deselect the parent `If` node
    const deselect = (event: Event) => {
      const parent = this.getIf()
      parent.deselect()
      event.stopPropagation()
    }

    const iconElem = html`<span
      class=${tw`flex items-center text-base mx-2 p-1 ${
        isActive
          ? `rounded-full border(& ${StencilaIf.color}-300) bg-${StencilaIf.color}-100`
          : ''
      }`}
    >
      <stencila-icon name=${iconName}></stencila-icon>
    </span>`

    const labelElem = html`<span class=${tw`mr-1 w-12`}>${label}</span>`

    const textEditor = html`<stencila-code-editor
      class=${tw`min-w-0 w-full rounded overflow-hidden border(& ${StencilaIf.color}-200)
                 bg-${StencilaIf.color}-50 font-normal
                 focus:border(& ${StencilaIf.color}-400) focus:ring(2 ${StencilaIf.color}-100)`}
      language=${this.programmingLanguage}
      single-line
      line-wrapping
      no-controls
      ?read-only=${readOnly}
      ?disabled=${readOnly}
      @focus=${deselect}
      @mousedown=${deselect}
      @stencila-document-patch=${(event: CustomEvent) => {
        const patch = event.detail.patch as Patch

        // If the `text` is currently empty and this is the last clause (i.e. and "else")
        // then request a rerender to make it an elif
        if (this.isLast) {
          // Set `text` to trigger update and recalculation of `isElse`
          const editor = event.target as StencilaCodeEditor
          this.text = editor.getCode()
        }

        // Emit patch using override above
        event.stopPropagation()
        this.emitPatch(patch)
      }}
      @stencila-ctrl-enter=${() => this.execute()}
    >
      <slot
        name="text"
        slot="code"
        @slotchange=${(event: Event) => this.onTextSlotChange(event)}
      ></slot>
    </stencila-code-editor>`

    const programmingLanguageMenu = html`<stencila-code-language
      class=${tw`ml-2 text(base gray-500)`}
      color=${StencilaIf.color}
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
        // Emit patch using override above
        event.stopPropagation()
        this.emitPatch(event.detail)
      }}
    ></stencila-code-language>`

    const moveUp = (event: KeyboardEvent) => {
      if (this.previousElementSibling) {
        this.parentElement?.insertBefore(this, this.previousElementSibling)
      }
      this.requestUpdateAll()

      this.emitOp({
        type: 'Move',
        from: [this.index],
        to: [this.index - 1],
        items: 1,
      })
    }

    const moveDown = (event: KeyboardEvent) => {
      if (this.nextElementSibling) {
        this.parentElement?.insertBefore(this.nextElementSibling, this)
      }
      this.requestUpdateAll()

      this.emitOp({
        type: 'Move',
        from: [this.index],
        to: [this.index + 1],
        items: 1,
      })
    }

    const moveButton = !readOnly
      ? html`<span
          class=${tw`flex justify-between items-center h-6 ml-2 rounded-full outline-none
                     bg-${StencilaIf.color}-200(hover:& focus:&) focus:ring(1 ${StencilaIf.color}-300)`}
          tabindex="0"
          @keydown=${(event: KeyboardEvent) => {
            const retainFocus = () => (event.target as HTMLElement).focus()
            if (event.key == 'ArrowUp' && !this.isFirst) {
              event.preventDefault()
              moveUp(event)
              retainFocus()
            } else if (event.key == 'ArrowDown' && !this.isLast) {
              event.preventDefault()
              moveDown(event)
              retainFocus()
            }
          }}
        >
          <stencila-icon
            name="arrow-up"
            class=${tw`w-3 ${
              this.isFirst ? 'text-gray-300' : 'cursor-n-resize'
            }`}
            ?aria-disabled=${this.isFirst}
            @click=${(event: KeyboardEvent) => !this.isFirst && moveUp(event)}
          ></stencila-icon>
          <stencila-icon
            name="arrow-down"
            class=${tw`w-3 ${
              this.isLast ? 'text-gray-300' : 'cursor-s-resize'
            }`}
            ?aria-disabled=${this.isLast}
            @click=${(event: KeyboardEvent) => !this.isLast && moveDown(event)}
          ></stencila-icon>
        </span>`
      : ''

    const remove = () => {
      this.emitOp({
        type: 'Remove',
        address: [],
        items: 1,
      })

      const parent = this.parentElement!

      this.remove()
      ;[...parent.children].forEach((clause: StencilaIfClause) =>
        clause.requestUpdate()
      )
    }

    const removeButton = !readOnly
      ? html`<stencila-icon-button
          name="x-circle"
          color=${StencilaIf.color}
          adjust="ml-2"
          @keydown=${(event: KeyboardEvent) =>
            event.key == 'Enter' && event.shiftKey && remove()}
          @click=${() => remove()}
        >
        </stencila-icon-button>`
      : ''

    const expandButton = html`<stencila-icon-button
      name="chevron-right"
      color=${StencilaIf.color}
      adjust=${`ml-2 rotate-${
        this.isExpanded ? '90' : '0'
      } transition-transform`}
      @click=${(event: KeyboardEvent) => {
        this.isExpanded = !this.isExpanded
        if (event.shiftKey) {
          const clauses = this.getIfClauses()
          clauses.forEach((clause) => (clause.isExpanded = this.isExpanded))
        }
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
        if (event.shiftKey) {
          const clauses = this.getIfClauses()
          clauses.forEach((clause) => (clause.isExpanded = this.isExpanded))
        }
      }}
    >
    </stencila-icon-button>`

    const errorsContainer = html`<div
      part="errors"
      class=${tw`border(t ${StencilaIf.color}-200) ${
        this.hasErrors || 'hidden'
      }`}
    >
      <slot
        name="errors"
        @slotchange=${(event: Event) => this.onErrorsSlotChange(event)}
      ></slot>
    </div>`

    return html`<div part="base" class=${tw`border(b ${StencilaIf.color}-200)`}>
      <div
        part="header"
        class=${tw`flex justify-between items-center bg-${StencilaIf.color}-50 p-1
                   font(mono bold) text(sm ${StencilaIf.color}-700)`}
        @mousedown=${toggleSelected}
      >
        ${iconElem} ${labelElem} ${textEditor} ${programmingLanguageMenu}
        ${moveButton} ${removeButton} ${expandButton}
      </div>
      ${errorsContainer} ${this.renderContentContainer()}
    </div>`
  }
}
