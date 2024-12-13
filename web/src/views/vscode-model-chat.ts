import { css, LitElement } from 'lit'
import { customElement } from 'lit/decorators'

import { ModelChatClient } from '../clients/model-chat'

import '../nodes'
import '../shoelace'

@customElement('stencila-vscode-model-chat-view')
export class VSCodeModelChatView extends LitElement {
  protected modelChatClient: ModelChatClient

  static override styles = css`
    :host {
      min-height: 100vh;
    }
  `

  protected override createRenderRoot() {
    this.modelChatClient = new ModelChatClient(this)
    return this
  }
}
