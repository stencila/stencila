import { html } from 'lit'
import { customElement } from 'lit/decorators'
import { css, apply } from 'twind/css'

import '@shoelace-style/shoelace/dist/components/icon/icon'
import '@shoelace-style/shoelace/dist/components/menu-item/menu-item'
import '@shoelace-style/shoelace/dist/components/select/select'

import { twSheet, varApply, varLocal } from '../utils/css'
import StencilaElement from './element'

const { tw, sheet } = twSheet()

@customElement('stencila-code-editor')
export default class StencilaCodeEditor extends StencilaElement {
  static styles = [sheet.target]

  render() {
    return html`<div
      class="${tw(
        css`
          ${varLocal(
            'border-style',
            'border-width',
            'border-color',
            'border-radius',
            'text-font',
            'text-size',
            'text-color'
          )}

          [part='language'] sl-select::part(control) {
            ${varApply(
              'border-style',
              'border-width',
              'border-color',
              'border-radius'
            )}
          }

          [part='language'] sl-select::part(control),
          [part='language'] sl-menu-item::part(display-label),
          [part='language'] sl-menu-item::part(prefix),
          [part='language'] sl-menu-item::part(label) {
            ${varApply('text-font', 'text-size', 'text-color')}
          }
        `
      )}"
    >
      <slot name="code"></slot>

      <slot name="messages"></slot>

      <div class="${tw`flex flex-row items-center justify-between`}">
        <div class="start">
          <slot name="info"></slot>
        </div>
        <div part="language" class="${tw`w-36`}">
          <sl-select size="small">
            <stencila-icon
              slot="prefix"
              name="code"
              label="Programming language"
            ></stencila-icon>
            <sl-menu-item value="python">
              <stencila-icon
                slot="prefix"
                name="lightning-fill"
                label="Executable"
                class="${tw`text-yellow-500`}"
              ></stencila-icon>
              Python
            </sl-menu-item>
            <sl-menu-item value="other">
              <stencila-icon
                slot="prefix"
                name="dash"
                label="Not executable"
              ></stencila-icon>
              Other
            </sl-menu-item>
          </sl-select>
        </div>
      </div>
    </div>`
  }
}
