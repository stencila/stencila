import { NodeType } from '@stencila/types'
import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { Executable } from './executable'

import '../ui/nodes/chat-message-inputs'
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
   * A list of node types that are initially expanded within a
   * model chat message
   */
  public static EXPANDED_NODE_TYPES: NodeType[] = [
    'CodeBlock',
    'CodeChunk',
    'Datatable',
    'Figure',
    'ForBlock',
    'IfBlock',
    'IncludeBlock',
    'InstructionBlock',
    'MathBlock',
    'RawBlock',
    'StyledBlock',
    'Table',
  ]

  override render() {
    const style = apply('min-w-[45ch] max-w-prose mx-auto')

    switch (this.messageRole) {
      case 'System':
        return this.renderSystemMessage(style)
      case 'User':
        return this.renderUserMessage(style)
      case 'Model':
        return this.renderModelMessage(style)
    }
  }

  renderSystemMessage(style: string) {
    return html`
      <div class="${style} my-3 p-3 bg-indigo-100 rounded">
        <slot name="content"></slot>
      </div>
    `
  }

  renderUserMessage(style: string) {
    return html`
      <div class="${style} flex justify-end">
        <div class="my-3 p-3 bg-blue-50 rounded w-content">
          <slot name="content"></slot>
          <slot name="files"></slot>
        </div>
      </div>
    `
  }

  renderModelMessage(style: string) {
    return html`<div class="${style} my-3 p-3">
      <slot name="author" class="text-blue-900"></slot>
      ${this.executionStatus === 'Running'
        ? this.renderRunningIndicator()
        : html`<slot name="content"></slot>`}
    </div>`
  }

  renderRunningIndicator() {
    const dotClasses = apply('h-2 w-2 bg-gray-500 rounded-full animate-bounce')

    return html`
      <div
        class="flex justify-center items-center gap-x-1 mt-3 p-5 rounded bg-gray-100 w-full"
      >
        <div class=${dotClasses} style="animation-delay: 0ms"></div>
        <div class=${dotClasses} style="animation-delay: 250ms"></div>
        <div class=${dotClasses} style="animation-delay: 500ms"></div>
      </div>
    `
  }
}
