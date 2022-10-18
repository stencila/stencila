import { html } from 'lit'
import { customElement, property, state } from 'lit/decorators'
import { twSheet } from '../utils/css'

import StencilaExecutable from './executable'

const { tw, sheet } = twSheet()

@customElement('stencila-for-iteration')
export default class StencilaForIteration extends StencilaExecutable {
  static styles = sheet.target

  /**
   * The index of this iteration within the parent `For` node
   */
  @property({ type: Number })
  index: number

  render() {
    return html`<div part="base">
      <div
        part="header"
        class=${tw`flex items-center border(t b blue-200) bg-blue-50 p-1 font(mono) text(sm blue-800)`}
      >
        <span class=${tw`ml-3`}>${this.index + 1}</span>
      </div>
      <div part="content" class=${tw`p-2`}>
        <slot></slot>
      </div>
    </div>`
  }
}
