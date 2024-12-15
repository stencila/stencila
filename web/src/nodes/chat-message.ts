import { html } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { documentCommandEvent } from '../clients/commands'
import { withTwind } from '../twind'
import '../ui/nodes/properties/authors'

import { Executable } from './executable'

/**
 * Web component representing a Stencila `ChatMessage` node
 *
 * Renders differently depending upon whether the messages is a system
 * message (i.e. a system prompt), a user message (i.e. an instruction
 * from the user), or a model message (i.e a response from a model).
 *
 * User messages are only editable, and only have a toolbar, if they
 * have not yet been "executed" successfully.
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/chat-message.md
 */
@customElement('stencila-chat-message')
@withTwind()
export class ChatMessage extends Executable {
  @property({ attribute: 'message-role' })
  messageRole: 'System' | 'User' | 'Model'

  @state()
  source: string = ''

  onSourceInput(event: Event) {
    const textarea = event.target as HTMLTextAreaElement

    // Update the height of the text area
    textarea.style.height = 'auto'
    textarea.style.height = `${Math.min(textarea.scrollHeight, 200)}px`

    // Update the message source
    this.source = textarea.value
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
    const nodeIds = [this.id]

    this.dispatchEvent(
      documentCommandEvent({
        command: 'patch-node-content',
        args: [this.id, this.source],
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
    switch (this.messageRole) {
      case 'System':
        return this.renderSystemMessage()
      case 'User':
        return this.renderUserMessage()
      case 'Model':
        return this.renderModelMessage()
    }
  }

  renderSystemMessage() {
    return html`<div class="my-3 rounded border border-gray-400 p-3">
      <slot name="content"></slot>
    </div>`
  }

  renderUserMessage() {
    return this.executionCount >= 0 ||
      ['Pending', 'Running', 'Succeeded'].includes(this.executionStatus)
      ? this.renderUserMessageInactive()
      : this.renderUserMessageActive()
  }

  renderUserMessageInactive() {
    return html`<div class="my-3 rounded border border-blue-400 p-3">
      <slot name="content"></slot>
    </div>`
  }

  renderUserMessageActive() {
    const hasContent = this.source.trim().length > 0

    return html`<div class="my-3 rounded border border-gray-200">
      <div class="flex items-end max-w-4xl mx-auto rounded p-2">
        <textarea
          class="w-full resize-none overflow-hidden outline-none px-2 py-1"
          placeholder=""
          rows=${1}
          @input=${(event: Event) => this.onSourceInput(event)}
        ></textarea>

        <sl-tooltip content=${hasContent ? 'Send message' : 'Message is empty'}
          ><stencila-ui-icon-button
            name="arrowNarrowUp"
            class=${hasContent ? 'text-blue-900' : 'text-grey-500'}
            ?disabled=${!hasContent}
            @click=${(event: Event) => this.onSend(event)}
          ></stencila-ui-icon-button
        ></sl-tooltip>
      </div>
    </div>`
  }

  renderModelMessage() {
    return html`<div class="my-3 rounded border border-red-400 p-3">
      <slot name="content"></slot>
    </div>`
  }
}
