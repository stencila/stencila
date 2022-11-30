import { html } from 'lit'
import { customElement } from 'lit/decorators'

import StencilaElement from '../utils/element'
import { twSheet } from '../utils/css'

const { tw, sheet } = twSheet()

@customElement('stencila-document-nav')
export default class StencilaDocumentNav extends StencilaElement {
  static styles = [sheet.target]

  render() {
    return html``
  }
}
