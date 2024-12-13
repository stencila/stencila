import { html, LitElement } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'
import { nodeUi } from '../ui/nodes/icons-and-colours'

import '../ui/icons/icon'

@customElement('stencila-model-chat')
@withTwind()
export class StencilaModelChat extends LitElement {
  protected override render() {
    const { colour, textColour } = nodeUi('InstructionBlock')

    return html`
      <div
        class="flex flex-col h-full px-6 bg-[${colour}] text-[${textColour}]"
      >
        <!-- header -->
        <div class="py-4">
          <div class="flex items-center">
            <stencila-ui-icon
              name="robot"
              class="mr-4 text-base"
            ></stencila-ui-icon>
            <h1 class="font-bold">Model Chat</h1>
          </div>
        </div>
        <!-- message feed -->
        <div class="h-full bg-white rounded">
          <slot name="message-feed"></slot>
        </div>
        <!-- instruction message -->
        <div>
          <slot name="instruction-message"></slot>
        </div>
      </div>
    `
  }
}
