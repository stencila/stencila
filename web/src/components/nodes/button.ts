import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { twSheet } from '../utils/css'
import StencilaExecutable from './executable'

const { tw, sheet } = twSheet()

@customElement('stencila-button')
export default class StencilaButton extends StencilaExecutable {
  static styles = [sheet.target]

  render() {
    return html`<sl-button @click=${() => this.execute()}
      ><slot></slot
    ></sl-button>`
  }
}
