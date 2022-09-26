import { html } from 'lit'
import { customElement } from 'lit/decorators'
import { apply as twApply, css } from 'twind/css'

import '@shoelace-style/shoelace/dist/components/menu-item/menu-item'
import '@shoelace-style/shoelace/dist/components/select/select'
import '@shoelace-style/shoelace/dist/components/tooltip/tooltip'

import '../base/icon'
import '../base/tag'
import '../editors/code-editor'
import { twSheet, varApply, varLocal } from '../utils/css'
import StencilaCodeExecutable from './code-executable'

const { tw, sheet } = twSheet()

@customElement('stencila-code-expression')
export default class StencilaCodeExpression extends StencilaCodeExecutable {
  static styles = [sheet.target]

  render() {
    return html`<span
      class="${tw(
        css`
          ${varLocal(
            'ui-border-style',
            'ui-border-width',
            'ui-border-color',
            'ui-border-radius',
            'ui-background-color',
            'ui-icon-color',
            'ui-font-family',
            'ui-font-size',
            'ui-text-color'
          )}

          ${varApply(
            'ui-border-style',
            'ui-border-width',
            'ui-border-color',
            'ui-border-radius',
            'ui-background-color',
            'ui-icon-color',
            'ui-font-family',
            'ui-font-size',
            'ui-text-color'
          )}

          ${twApply('inline-block my-1 py-0.5 px-1')}
        `
      )}"
    >
      ${this.renderExecuteIcon(tw)}
      <stencila-tag color="green">${this.id}</stencila-tag>
      ${this.renderLanguageSelector(tw)}
      <stencila-code-editor
        part="code"
        language=${this.programmingLanguage}
        ?read-only=${!this.isEditable()}
        single-line
        line-wrapping
        no-controls
        @ctrl-enter=${this.execute}
        class="${this.isCodeVisible ? '' : tw`hidden`}"
      >
        <slot name="text" slot="code"></slot>
      </stencila-code-editor>
    </span>`
  }
}
