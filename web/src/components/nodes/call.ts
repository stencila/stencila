import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { twSheet } from '../utils/css'
import StencilaInclude from './include'

const { tw, sheet } = twSheet()

@customElement('stencila-call')
export default class StencilaCall extends StencilaInclude {
  static styles = [sheet.target]

  render() {
    return html`<span
      ><stencila-tag color="blue">${this.id}</stencila-tag></span
    >`
  }
}
