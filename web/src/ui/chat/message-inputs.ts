import { html, LitElement } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { documentCommandEvent } from '../../clients/commands'
import { withTwind } from '../../twind'
import { NodeId } from '../../types'

import '../inputs/file-input'

@customElement('stencila-chat-message-inputs')
@withTwind()
export class MessageInput extends LitElement {
  @property({ type: String, attribute: 'message-id' })
  messageId: NodeId

  @state()
  source: string = ''

  private tempFileUrls: string[] = []

  private files: File[] = []

  onSourceInput(event: Event) {
    const textarea = event.target as HTMLTextAreaElement

    // Update the height of the text area
    textarea.style.height = 'auto'
    textarea.style.height = `${Math.min(textarea.scrollHeight, 200)}px`

    // Update the message source
    this.source = textarea.value
  }

  private onFileInputChange(e: Event) {
    const files = (e.target as HTMLInputElement).files
    this.files.push(...files)
  }

  onSourceKeyDown(event: KeyboardEvent) {
    switch (event.key) {
      case 'Enter':
        if (event.shiftKey) {
          return
        }
        event.preventDefault()
        this.onSend(event)
        return
      default:
        return
    }
  }

  onPaste(event: ClipboardEvent) {
    const clipboardData = event.clipboardData

    const hasFiles = clipboardData.types.includes('Files')

    // If files are present, handle custom paste
    if (hasFiles) {
      event.preventDefault()
      const textarea = event.target as HTMLTextAreaElement

      Array.from(clipboardData.items).forEach((item) => {
        if (item.kind === 'file') {
          const pastedFile = item.getAsFile()

          // add to component files
          this.files.push(pastedFile)

          // TODO: find a way to get the
          // Create a temporary URL for the file
          const fileURL = URL.createObjectURL(pastedFile)

          // Get the current cursor position
          const startPos = textarea.selectionStart
          const endPos = textarea.selectionEnd
          const currentValue = textarea.value

          // Create Markdown image link
          const markdownImageLink = `![${pastedFile.name}](${fileURL})`

          // Insert the Markdown link at the cursor position
          textarea.value =
            currentValue.substring(0, startPos) +
            markdownImageLink +
            currentValue.substring(endPos)

          // Move cursor to the end of the inserted link
          textarea.selectionStart = textarea.selectionEnd =
            startPos + markdownImageLink.length

          this.tempFileUrls.push(fileURL)
        }
      })
    }
  }

  /**
   * Send a user message to the chat
   *
   * Patches the content of the chat (the server has a custom patch handler to
   * convert Markdown content to a vector of blocks) and executes the chat message
   * (which executes the entire chat).
   */
  onSend(event: Event) {
    event.stopImmediatePropagation()

    const nodeType = 'ChatMessage'
    const nodeIds = [this.messageId]

    this.dispatchEvent(
      documentCommandEvent({
        command: 'patch-node-format',
        args: [this.id, 'content', 'md', this.source, 'block'],
      })
    )

    this.dispatchEvent(
      documentCommandEvent({
        command: 'execute-nodes',
        nodeType,
        nodeIds,
      })
    )
  }

  override render() {
    const hasContent = this.source.trim().length > 0

    return html`
      <div class="my-3 rounded border border-gray-500">
        <div class="max-w-4xl mx-auto rounded p-2">
          <textarea
            class="w-full resize-none overflow-hidden outline-none px-1 py-1"
            placeholder=""
            rows=${1}
            @input=${(event: Event) => this.onSourceInput(event)}
            @keydown=${this.onSourceKeyDown}
            @paste=${this.onPaste}
          ></textarea>
          <div class="flex items-center justify-between">
            <div>
              <sl-tooltip content=${'Add file'}>
                <stencila-ui-file-input
                  .fileChangeHandler=${this.onFileInputChange}
                  class="text-blue-900"
                ></stencila-ui-file-input>
              </sl-tooltip>
            </div>
            <sl-tooltip
              content=${hasContent ? 'Send message' : 'Message is empty'}
              ><stencila-ui-icon-button
                name="arrowNarrowUp"
                class=${hasContent ? 'text-blue-900' : 'text-grey-500'}
                ?disabled=${!hasContent}
                @click=${(event: Event) => this.onSend(event)}
              ></stencila-ui-icon-button
            ></sl-tooltip>
          </div>
        </div>
      </div>
    `
  }

  override disconnectedCallback(): void {
    this.tempFileUrls.forEach((url) => {
      URL.revokeObjectURL(url)
    })
  }
}
