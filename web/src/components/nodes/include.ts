import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { twSheet } from '../utils/css'
import StencilaExecutable from './executable'

const { tw, sheet } = twSheet()

@customElement('stencila-include')
export default class StencilaInclude extends StencilaExecutable {
  static styles = [sheet.target]

  render() {
    return html`<span
      ><stencila-tag color="indigo">${this.id}</stencila-tag></span
    >`
  }
}
