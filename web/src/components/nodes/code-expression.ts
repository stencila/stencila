import { sentenceCase } from 'change-case'
import { html } from 'lit'
import { customElement, property, state } from 'lit/decorators'
import { apply as twApply, css } from 'twind/css'

import '@shoelace-style/shoelace/dist/components/menu-item/menu-item'
import '@shoelace-style/shoelace/dist/components/select/select'
import '@shoelace-style/shoelace/dist/components/tooltip/tooltip'

import '../base/icon'
import '../base/tag'
import '../editors/code-editor'
import { twSheet, varApply, varLocal, varPass } from '../utils/css'
import StencilaCodeExecutable from './code-executable'

const { tw, sheet } = twSheet()

@customElement('stencila-code-expression')
export default class StencilaCodeExpression extends StencilaCodeExecutable {
  static styles = [sheet.target]

  render() {
    return html`<span
      ><stencila-tag color="green">${this.id}</stencila-tag></span
    >`
  }
}
