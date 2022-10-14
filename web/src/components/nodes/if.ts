import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { twSheet } from '../utils/css'
import StencilaExecutable from './executable'
import './if-clause'

const { tw, sheet } = twSheet()

/**
 * A component representing a Stencila `If` document node
 */
@customElement('stencila-if')
export default class StencilaIf extends StencilaExecutable {
  static styles = sheet.target

  protected render() {
    return html`<div
      part="base"
      class=${tw`my-4 rounded border(& violet-200) overflow-hidden`}
    >
      <div part="clauses">
        <slot name="clauses"></slot>
      </div>
    </div>`
  }
}
