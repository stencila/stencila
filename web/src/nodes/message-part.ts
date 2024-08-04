import { apply } from '@twind/core'
import { html, LitElement } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'
import { nodeUi } from '../ui/nodes/icons-and-colours'

@customElement('stencila-message-part')
@withTwind()
export class MessagePart extends LitElement {
  @property()
  type: 'text' | 'image'

  @property()
  value: string

  override render() {
    return html`<div class="px-3 py-1.5">
      ${this.type === 'image' ? this.renderImage() : this.renderText()}
    </div>`
  }

  renderText() {
    const { textColour, borderColour } = nodeUi('InstructionBlock')

    const textAreaStyles = apply([
      'w-full m-auto resize-none',
      `rounded-sm border border-[${borderColour}]`,
      `outline-[${textColour}]/50`,
      'px-2 py-1.5',
      'font-sans text-gray-700 text-sm',
    ])

    return html`<div class="flex items-start">
      <sl-icon name="chat-square" class="mr-2"></sl-icon>
      <textarea class=${textAreaStyles}>${this.value}</textarea>
    </div>`
  }

  renderImage() {
    const { borderColour } = nodeUi('InstructionBlock')

    const divStyles = apply([
      'w-full',
      `rounded-sm border border-[${borderColour}]`,
      'bg-white',
      'p-2',
    ])

    return html`<div class="flex items-start">
      <sl-icon name="image" class="mr-2"></sl-icon>
      <div class=${divStyles}>
        <img class="max-w-md mx-auto" src=${this.value} />
      </div>
    </div>`
  }
}
