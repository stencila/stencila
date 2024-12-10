import { html, LitElement } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../../twind'
import { nodeUi } from '../nodes/icons-and-colours'

@customElement('stencila-ui-model-chat-header')
@withTwind()
export class UIChatAssistantPanelHeader extends LitElement {
  override render() {
    const { borderColour, colour, textColour } = nodeUi('InstructionBlock')
    return html`<div
      class="p-4 bg-[${colour}] text-[${textColour}] border-b border-[${borderColour}]"
    >
      <h1>I am a happy little chat window</h1>
    </div>`
  }
}
