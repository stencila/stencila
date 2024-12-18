import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { Executable } from './executable'

import '../ui/chat/message-inputs'
import '../ui/animation/logo'

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

  /**
   * A public list of node cards to expand automatically in a model chat message
   */
  public static DEFAULT_EXPANDED_NODE_CARDS = [
    'CodeExpression',
    'CodeChunk',
    'CodeBlock',
    'Figure',
    'ForBlock',
    'IncludeBlock',
    'IfBlock',
    'MathBlock',
    'Datatable',
    'Table',
    'StyledBlock',
    'RawBlock',
    'InstructionBlock', // < ?
  ]

  /**
   * Whether the message has any content
   *
   * Used to determine whether to render inputs for user messages
   * and placeholder text for model messages while running.
   */
  private hasContent(): boolean {
    // The `<div slot=content>` is the only present if content is not empty
    return this.querySelector('div[slot=content]') !== null
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
    return html`
      <div class="my-3 p-3 bg-blue-100/50 rounded">
        <slot name="content"></slot>
      </div>
    `
  }

  renderUserMessage() {
    return this.hasContent()
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
    const containerStyles = apply([
      'fixed bottom-0 left-0 z-10',
      'w-full',
      'bg-gray-100',
      'border-t border-black/20',
    ])

    return html`
      <div class=${containerStyles}>
        <div class="max-w-[400px] mx-auto">
          <stencila-chat-message-inputs
            message-id=${this.id}
          ></stencila-chat-message-inputs>
        </div>
      </div>
    `
  }

  renderModelMessage() {
    return html`
      <div class="my-3 p-3">
        ${this.hasContent()
          ? ''
          : html`
              <div class="text-4xl flex justify-center items-center">
                <stencila-animated-logo></stencila-animated-logo>
              </div>
            `}
        <slot name="content"></slot>
      </div>
    `
  }
}
