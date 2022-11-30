import { html } from 'lit'
import { customElement } from 'lit/decorators'

import StencilaElement from '../utils/element'
import { twSheet } from '../utils/css'

const { tw, sheet } = twSheet()

@customElement('stencila-document-toc')
export default class StencilaDocumentToc extends StencilaElement {
  static styles = [sheet.target]

  render() {
    return html`<nav></nav>`
  }
}
