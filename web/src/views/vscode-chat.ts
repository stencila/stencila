import { css, html, LitElement } from 'lit'
import { customElement } from 'lit/decorators'

import { ChatAssistantClient } from '../clients/chatview'
import { withTwind } from '../twind'
import { nodeUi } from '../ui/nodes/icons-and-colours'

import '../nodes'
import '../shoelace'

@customElement('stencila-vscode-chat-assistant-view')
@withTwind()
export class VSCodeAssistantView extends LitElement {
  protected chatAssistantClient: ChatAssistantClient

  static override styles = css`
    :host {
      min-height: 100vh;
    }
  `

  override connectedCallback(): void {
    super.connectedCallback()
    this.chatAssistantClient = new ChatAssistantClient(this)
  }

  override render() {
    const { colour, borderColour, textColour } = nodeUi('InstructionBlock')

    return html`
      <div class="flex flex-col w-full h-full font-sans">
        <div
          class="p-4 bg-[${colour}] text-[${textColour}] border-b border-[${borderColour}]"
        >
          <h1>I am a happy little chat window</h1>
        </div>
        <div class="flex-grow bg-white my-4 p-4 overflow-y-auto">
          <slot></slot>
        </div>
        <div class="bg-[${colour}] border-t border-[${borderColour}] p-4">
          <textarea type="text" class="outline-none"></textarea>
        </div>
      </div>
    `
  }
}
