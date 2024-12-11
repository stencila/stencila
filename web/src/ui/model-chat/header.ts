import { html, LitElement } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../twind'
import { nodeUi } from '../nodes/icons-and-colours'

import '../icons/icon'

@customElement('stencila-ui-model-chat-header')
@withTwind()
export class UIChatAssistantPanelHeader extends LitElement {
  @property()
  currentModel: string

  override render() {
    const { borderColour, colour, textColour } = nodeUi('InstructionBlock')
    return html`
      <div
        class="p-4 bg-[${colour}] text-[${textColour}] border-b border-[${borderColour}]"
      >
        <div class="flex items-center">
          <stencila-ui-icon
            name="robot"
            class="mr-4 text-base"
          ></stencila-ui-icon>
          <h1 class="font-bold">Model Chat</h1>
        </div>
      </div>
    `
  }
}
