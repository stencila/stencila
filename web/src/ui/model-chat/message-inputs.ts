import { html, LitElement } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../../twind'

import '../inputs/file-input'
import './prosemirror-input'

@customElement('stencila-message-input')
@withTwind()
export class MessageInput extends LitElement {
  @property({ type: Boolean })
  pending: boolean = false

  @property({ type: String })
  currentModel: string

  private files: File[] = []
  private textAreaValue: string = ''

  private handleSend() {
    this.dispatchEvent(
      new CustomEvent('stencila-message-send', {
        detail: {
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

  private fileInputHandler(e: Event) {
    const files = (e.target as HTMLInputElement).files
    this.files.push(...files)
  }

  override render() {
    return html`
      <div class="py-4 border rounded">
        <stencila-ui-filedrop-wrapper .dropEvent=${this.fileDropHandler}>
          <div class="p-1 flex gap-4 items-center">
            <textarea
              class="w-full h-16 p-1 rounded resize-none outline-none"
              @keydown=${(e: KeyboardEvent) => {
                if (e.key === 'Enter') {
                  e.preventDefault()
                  this.handleSend()
                }
              }}
              @change=${(e: Event) => {
                this.textAreaValue = (e.target as HTMLTextAreaElement).value
              }}
              ?disabled=${this.pending}
              .fileChangeHandler=${this.fileInputHandler}
              placeholder="message ${this.currentModel}"
            ></textarea>
            <stencila-ui-file-input> </stencila-ui-file-input>
            <button
              class="p-4 bg-blue-500 text-white text-sm border border-blue-500 rounded cursor-pointer hover:border-white"
              @click=${this.handleSend}
              ?disabled=${this.pending}
            >
              Send
            </button>
          </div>
        </stencila-ui-filedrop-wrapper>
      </div>
    `
  }
}
