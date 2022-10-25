import HtmlFragment from 'html-fragment'
import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { twSheet } from '../utils/css'
import StencilaExecutable from './executable'
import './if-clause'
import StencilaIfClause from './if-clause'

const { tw, sheet } = twSheet()

/**
 * A component representing a Stencila `If` document node
 */
@customElement('stencila-if')
export default class StencilaIf extends StencilaExecutable {
  static styles = sheet.target

  static color = 'blue'

  static formats = ['markdown', 'yaml', 'json']

  protected render() {
    const readOnly = this.isReadOnly()

    const add = () => {
      const clauses = (
        this.renderRoot.querySelector('slot[name=clauses]') as HTMLSlotElement
      ).assignedElements({
        flatten: true,
      })[0]

      this.emitOps({
        type: 'Add',
        address: ['clauses', clauses.childElementCount],
        length: 1,
        value: StencilaIfClause.json,
      })

      clauses.appendChild(HtmlFragment(StencilaIfClause.html))
      ;[...clauses.children].forEach((clause: StencilaIfClause) =>
        clause.requestUpdate()
      )
    }

    const addButton = !readOnly
      ? html`<stencila-icon-button
          name="plus-lg"
          color=${StencilaIf.color}
          adjust="ml-3"
          @keydown=${(event: KeyboardEvent) => event.key == 'Enter' && add()}
          @click=${() => add()}
        >
        </stencila-icon-button>`
      : html`<span></span>`

    return html`<div
      part="base"
      class=${tw`my-4 rounded border(& ${StencilaIf.color}-200) overflow-hidden`}
    >
      <div part="clauses">
        <slot name="clauses"></slot>
      </div>

      <div
        part="footer"
        class=${tw`flex justify-between items-center bg-${StencilaIf.color}-50 p-1
                  font(mono bold) text(sm ${StencilaIf.color}-700)`}
      >
        ${addButton}
        ${this.renderEntityDownload(StencilaIf.formats, StencilaIf.color)}
      </div>
    </div>`
  }
}
