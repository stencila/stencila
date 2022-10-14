import { html } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { Patch } from '../../types'
import '../editors/code-editor'
import { twSheet } from '../utils/css'
import StencilaExecutable from './executable'

const { tw, sheet } = twSheet()

/**
 * A component representing a Stencila `IfClause` document node
 */
@customElement('stencila-if-clause')
export default class StencilaIfClause extends StencilaExecutable {
  static styles = sheet.target

  /**
   * An expression, in the programming language, that evaluates to a truthy/falsy value
   */
  @property()
  text: string

  /**
   * The programming language of the expression
   */
  @property({ attribute: 'programming-language' })
  programmingLanguage: string

  /**
   * Whether the clause is active (the first amongst the clauses to be truthy)
   */
  @property({ type: Boolean, attribute: 'is-active' })
  private isActive: boolean

  /**
   * Whether the content of the clause is visible to the user
   */
  @state()
  private isExpanded: boolean = false

  /**
   * Get the parent `If` element
   */
  private getIf() {
    return this.parentElement!.parentElement! as HTMLElement
  }

  /**
   * Get all the clauses in the parent `If` element
   */
  private getIfClauses() {
    return [...this.parentElement!.children] as StencilaIfClause[]
  }

  /**
   * Override of `Entity.sendPatch` to use targe the parent `If` node by using
   * the id of the containing <stencila-if> node and prepending the address
   * for this clause
   */
  protected async sendPatch(patch: Patch) {
    const index = this.getIfClauses().indexOf(this)

    const ops = patch.ops.map((op) => {
      if (op.type === 'Move')
        throw new Error('Unable to adjust a Move operation')
      return {
        ...op,
        address: ['clauses', index, ...op.address],
      }
    })

    return super.sendPatch({
      target: this.getIf().id,
      ops,
    })
  }

  /**
   * Override of `Executable.execute` to execute the parent `If` node by using
   * the id of the containing <stencila-if> node
   */
  protected execute() {
    this.emit('stencila-document-execute', {
      nodeId: this.getIf().id,
      ordering: 'Single',
    })
  }

  protected render() {
    const clauses = this.getIfClauses()
    const index = clauses.indexOf(this)
    const isLast = index == clauses.length - 1
    const label =
      index == 0 ? 'if' : isLast && this.text.length == 0 ? 'else' : 'elif'
    const icon =
      label == 'if' || label == 'elif' ? 'arrow-right' : 'arrow-return-right'

    const text =
      label !== 'else'
        ? html`<stencila-code-editor
            class=${tw`min-w-0 w-full rounded overflow-hidden border(& violet-200) focus:ring(1 violet-200) bg-violet-50`}
            language=${this.programmingLanguage}
            single-line
            line-wrapping
            no-controls
            @stencila-patch=${(event: CustomEvent) =>
              this.sendPatch(event.detail)}
            @stencila-ctrl-enter=${() => this.execute()}
          >
            <code slot="code">${this.text}</code>
          </stencila-code-editor>`
        : html`<span class=${tw`min-w-0 w-full`}></span>`

    return html`<div part="base">
      <div part="header" class=${tw`flex justify-between items-center ${
        isLast && !this.isExpanded ? '' : 'border(b violet-200)'
      } bg-violet-50 p-1 font(mono bold) text(sm violet-800)`}>
        <span class=${tw`flex items-center text-base mx-2`}>
        <stencila-icon name=${icon}></stencila-icon>
        </span>
        <span class=${tw`mr-1 w-12`}>${label}</span>
        ${text}
        <span>
            <sl-tooltip style="--show-delay: 500ms">
                <span class=${tw`text-xs`} slot="content">
                Click to ${
                  this.isExpanded ? 'hide' : 'show'
                } content for this clause.<br>
                <kbd>Shift</kbd>+click to ${
                  this.isExpanded ? 'hide' : 'show'
                } content for all clauses
                </span>
                <stencila-icon-button class=${tw`ml-1 rotate-${
                  this.isExpanded ? '90' : '0'
                } transition-transform`} name=${`chevron-right`} @click=${(
      event: KeyboardEvent
    ) => {
      this.isExpanded = !this.isExpanded
      if (event.shiftKey) {
        clauses.forEach((sibling) => (sibling.isExpanded = this.isExpanded))
      }
    }}></stencila-icon></sl-tooltip>
        </span>
      </div>

      <div part="content" class=${tw`p-2 ${
        isLast ? '' : 'border(b violet-200)'
      } ${this.isActive || this.isExpanded || 'hidden'}`}>
        <slot name="content"></slot>
      </div>
    </div>`
  }
}
