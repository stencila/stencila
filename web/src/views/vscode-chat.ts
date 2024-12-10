import { css, html, LitElement } from 'lit'
import { customElement } from 'lit/decorators'

import { ChatAssistantClient } from '../clients/chat-assistant'
import { withTwind } from '../twind'

import '../nodes'
import '../shoelace'
import '../ui/chat-assistant/user-input'
import '../ui/chat-assistant/header'

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
    return html`
      <div class="flex flex-col w-full h-full font-sans">
        <stencila-ui-assistant-panel-header>
        </stencila-ui-assistant-panel-header>
        <div class="flex-grow bg-white my-4 p-4 overflow-y-auto">
          <slot></slot>
        </div>
        <ui-chat-assist-user-inputs></ui-chat-assist-user-inputs>
      </div>
    `
  }
}
