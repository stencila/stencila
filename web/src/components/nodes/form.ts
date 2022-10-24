import { html } from 'lit'
import { customElement } from 'lit/decorators'
import { twSheet } from '../utils/css'

import StencilaExecutable from './executable'

const { tw, sheet } = twSheet()

/**
 * A component for a Stencila `Form` node
 *
 *
 */
@customElement('stencila-form')
export default class StencilaForm extends StencilaExecutable {
  static styles = [sheet.target]

  render() {
    return html`<form class=${tw`rounded border(& gray-100) p-2`}>
      <sl-button @click=${() => this.execute('Single')}>Refresh</sl-button>
      <slot name="errors"></slot>
      <slot name="content"></slot>
      <sl-button @click=${() => this.execute('Single')}>Submit</sl-button>
    </form>`
  }
}
