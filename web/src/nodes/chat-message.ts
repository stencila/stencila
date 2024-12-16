import { css, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { Executable } from './executable'

import '../ui/nodes/properties/authors'
import '../ui/chat/message-inputs'

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
    return html`
      <div class="my-3 p-3 bg-blue-100/50 rounded">
        <slot name="content"></slot>
      </div>
    `
  }

  renderUserMessage() {
    return this.executionCount >= 0 ||
      ['Pending', 'Running', 'Succeeded'].includes(this.executionStatus)
      ? this.renderUserMessageInactive()
      : this.renderUserMessageActive()
  }

  renderUserMessageInactive() {
    return html`
      <div class="flex justify-end">
        <div class="my-3 p-3 bg-green-100/50 rounded w-content">
          <slot name="content"></slot>
        </div>
      </div>
    `
  }

  renderUserMessageActive() {
    return html`
      <stencila-chat-message-inputs
        message-id=${this.id}
      ></stencila-chat-message-inputs>
    `
  }

  renderModelMessage() {
    return html`<div class="my-3 p-3">
      <slot name="content"></slot>
    </div>`
  }
}
