import { css, html } from 'lit'
import { customElement } from 'lit/decorators'
import { TW } from 'twind'

import { twSheet } from '../utils/css'
import './call-argument'
import StencilaInclude from './include'

const { tw, sheet } = twSheet()

/**
 * A component representing a Stencila `Call` node
 *
 * @slot arguments The `Call.arguments` property
 */
@customElement('stencila-call')
export default class StencilaCall extends StencilaInclude {
  static styles = [
    sheet.target,
    css`
      sl-input::part(base) {
        font-family: monospace;
      }
    `,
  ]

  static color = 'blue'

  static formats = ['markdown', 'yaml', 'json']

  protected renderArgumentsContainer(tw: TW) {
    return html`<div
      part="arguments"
      class=${tw`${this.isExpanded || 'hidden'}`}
    >
      <slot name="arguments"></slot>
    </div>`
  }

  protected render() {
    return html`<div
      part="base"
      class=${tw`my-4 rounded border(& ${StencilaCall.color}-200) overflow-hidden`}
    >
      <div
        part="header"
        class=${tw`flex items-center bg-${StencilaCall.color}-50 p-1 font(mono bold) text(sm ${StencilaCall.color}-600)`}
      >
        <span class=${tw`flex items-center text-base ml-1 mr-2`}>
          <stencila-icon name="call-outgoing"></stencila-icon>
        </span>
        <span class=${tw`mr-2`}>call</span>
        ${this.renderSourceInput(tw, 'execute')}
        <span class=${tw`mx-2`}>select</span>
        ${this.renderSelectInput(tw, 'execute')}
        ${this.renderExpandButton(tw, StencilaCall.color)}
      </div>

      ${this.renderArgumentsContainer(tw)}
      ${this.renderErrorsContainer(tw, StencilaCall.color)}
      ${this.renderContentContainer(tw, StencilaCall.color)}

      <div
        part="footer"
        class=${tw`grid justify-items-end items-center bg-${StencilaCall.color}-50
                       border(t ${StencilaCall.color}-200) p-1 text(sm ${StencilaCall.color}-600)`}
      >
        ${this.renderDownloadButton(StencilaCall.formats, StencilaCall.color)}
      </div>
    </div>`
  }
}
