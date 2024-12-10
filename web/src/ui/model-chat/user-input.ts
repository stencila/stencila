import { html, LitElement } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { withTwind } from '../../twind'
import { nodeUi } from '../nodes/icons-and-colours'

import '../inputs/filedrop-wrapper'

@customElement('stencila-ui-model-chat-user-inputs')
@withTwind()
export class UIModelChatUserInputs extends LitElement {
  /**
   * Boolean property
   */
  @property({ type: Boolean })
  waiting: boolean = false

  /**
   * Array of files added by user
   */
  @state()
  public files: File[] = []

  /**
   * Captured value of `textarea` input.
   */
  @state()
  public textAreaValue: string = ''

  private handleSend() {
    this.dispatchEvent(
      new CustomEvent('stencila-model-chat-command', {
        detail: {
          command: 'send-chat-message',
          text: this.textAreaValue,
          files: this.files,
        },
        bubbles: true,
        composed: true,
      })
    )
  }

  private fileDropHandler(e: DragEvent) {
    const files = e.dataTransfer.files
    this.files.push(...files)
  }

  override render() {
    const { borderColour, colour } = nodeUi('InstructionBlock')

    return html`
      <div class="bg-[${colour}] p-4 border-t border-[${borderColour}]">
        <stencila-ui-filedrop-wrapper .dropEvent=${this.fileDropHandler}>
          <div class="p-1 flex gap-4 items-center">
            <textarea
              class="w-full h-16 p-1 rounded resize-none outline-[${borderColour}]"
              @keydown=${(e: KeyboardEvent) => {
                if (e.key === 'Enter') {
                  e.preventDefault()
                  this.handleSend()
                }
              }}
              @change=${(e: Event) => {
                this.textAreaValue = (e.target as HTMLTextAreaElement).value
              }}
              ?disabled=${this.waiting}
              .value=${this.textAreaValue}
            ></textarea>
            <button
              class="p-4 bg-blue-500 text-white border border-blue-500 rounded cursor-pointer hover:border-white"
              @click=${this.handleSend}
              ?disabled=${this.waiting}
            >
              Send
            </button>
          </div>
        </stencila-ui-filedrop-wrapper>
      </div>
    `
  }
}
