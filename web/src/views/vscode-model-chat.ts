import { css, html, LitElement } from 'lit'
import { customElement } from 'lit/decorators'

import { ModelChatClient } from '../clients/model-chat'
import { withTwind } from '../twind'

import '../nodes'
import '../shoelace'
import '../ui/model-chat/user-input'
import '../ui/model-chat/header'

@customElement('stencila-vscode-model-chat-view')
@withTwind()
export class VSCodeModelChatView extends LitElement {
  protected modelChatClient: ModelChatClient

  static override styles = css`
    :host {
      min-height: 100vh;
    }
  `

  override connectedCallback(): void {
    super.connectedCallback()
    this.modelChatClient = new ModelChatClient(this)
  }

  override render() {
    return html`
      <div class="flex flex-col w-full h-full font-sans">
        <stencila-ui-model-chat-header></stencila-ui-model-chat-header>
        <div class="flex-grow bg-white my-4 p-4 overflow-y-auto">
          <slot></slot>
        </div>
        <stencila-ui-model-chat-user-inputs></stencila-ui-model-chat-user-inputs>
      </div>
    `
  }
}
