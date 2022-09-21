import { html } from 'lit'
import { customElement } from 'lit/decorators'

import '@shoelace-style/shoelace/dist/components/button/button'
import '@shoelace-style/shoelace/dist/components/dropdown/dropdown'
import '@shoelace-style/shoelace/dist/components/menu-item/menu-item'
import '@shoelace-style/shoelace/dist/components/menu/menu'

import { Mode, modeDesc, modeToString } from '../../mode'
import StencilaElement from '../base/element'
import { IconName } from '../base/icon'
import { twSheet } from '../utils/css'

const { tw, sheet } = twSheet()

@customElement('stencila-document-header')
export default class StencilaDocumentHeader extends StencilaElement {
  static styles = [sheet.target]

  render() {
    return html`
      <nav
        class="${tw`fixed top-0 left-0 right-0 bg-white shadow-md p-2 flex flex-row items-center justify-between`}"
      >
        <div part="start"></div>
        <div part="middle"></div>
        <div part="end">
          <sl-dropdown>
            <sl-button slot="trigger" caret>View</sl-button>
            <sl-menu>
              ${[Mode.Static, Mode.Read, Mode.Interact, Mode.Inspect].map(
                (mode) =>
                  html`<a href="?mode=${modeToString(mode).toLowerCase()}"
                    ><sl-menu-item>
                      <stencila-icon
                        slot="prefix"
                        name="${modeIcon(mode)}"
                      ></stencila-icon>
                      ${modeDesc(mode)}
                    </sl-menu-item></a
                  >`
              )}
            </sl-menu>
          </sl-dropdown>
        </div>
      </nav>
    `
  }
}

function modeIcon(mode: Mode): IconName {
  switch (mode) {
    case Mode.Static:
      return 'hourglass'
    default:
      return 'clock'
  }
}
