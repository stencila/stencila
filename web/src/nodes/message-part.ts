import { apply } from '@twind/core'
import { html, LitElement, PropertyValues } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { withTwind } from '../twind'
import { nodeUi } from '../ui/nodes/icons-and-colours'

@customElement('stencila-message-part')
@withTwind()
export class MessagePart extends LitElement {
  @property()
  type: 'text' | 'image'

  @state()
  instructionText: string

  @state()
  imgSrc: string

  protected override firstUpdated(_changedProperties: PropertyValues): void {
    super.firstUpdated(_changedProperties)
  }

  protected override render() {
    if (this.type === 'image') {
      return this.renderImage()
    } else {
      return this.renderText()
    }
  }

  renderText() {
    const { borderColour } = nodeUi('InstructionBlock')
    const styles = apply([
      'h-12 w-full',
      'px-1 mt-2',
      'text-black text-sm',
      `border border-[${borderColour}] rounded-sm`,
      'outline-black',
      'resize-none',
    ])

    // extract text from hidden slot
    return html`
      <div class="text-xs w-full">
        <label>Instruction Text:</label>
        <textarea class=${styles}>${this.instructionText}</textarea>
      </div>
      <div hidden>
        <slot
          @slotchange=${(e: Event) => {
            // @ts-expect-error Text node has data property
            this.instructionText = e.target.assignedNodes()[0].data
          }}
        ></slot>
      </div>
    `
  }

  renderImage() {
    // hide slot content to avoid default image styles
    return html`
      <div class="text-xs w-16 h-auto">
        <label>img: </label>
        <sl-tooltip content=${this.imgSrc}>
          <img src=${this.imgSrc} class="w-full mt-2" />
        </sl-tooltip>
        <div hidden>
          <slot
            @slotchange=${(e: Event) => {
              const img = (e.target as HTMLSlotElement).assignedElements()[0]
              if (img.tagName.toLowerCase() === 'img') {
                this.imgSrc = (img as HTMLImageElement).src
              }
            }}
          ></slot>
        </div>
      </div>
    `
  }
}
