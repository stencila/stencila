import { html } from 'lit'
import { customElement } from 'lit/decorators'

import StencilaElement from '../base/element'
import { twSheet } from '../utils/css'

const { tw, sheet } = twSheet()

@customElement('stencila-document-footer')
export default class StencilaDocumentFooter extends StencilaElement {
  static styles = [sheet.target]

  render() {
    return html`<nav></nav>`
  }
}
