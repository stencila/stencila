import { NodeType } from '@stencila/types'
import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { documentCommandEvent } from '../clients/commands'
import { withTwind } from '../twind'
import { closestGlobally } from '../utilities/closestGlobally'

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

  @property({
    attribute: 'is-selected',
    type: Boolean,
    // Converter needed because encoded not a boolean attribute (present or absent)
    // but as a stringified boolean
    converter: (attr) => attr == 'true',
  })
  isSelected: boolean = false

  /**
   * When the message is selected send a patch to the message group.
   * In Rust, this is handled by a custom patch operation handler so that only one
   * message is ever selected.
   */
  private onSelected() {
    const group = this.closestGlobally('stencila-chat-message-group')
    if (!group) {
      return
    }

    // Set `isSelected` on all siblings (and this) and determine the
    // index of this in the messages for the patch
    let thisIndex
    Array.from(this.parentNode.children).forEach(
      (message: ChatMessage, index) => {
        const selected = message.isSameNode(this)
        message.isSelected = selected
        if (selected) {
          thisIndex = index
        }
      }
    )

    this.dispatchEvent(
      documentCommandEvent({
        command: 'patch-node',
        args: [
          'ChatMessageGroup',
          group.id,
          ['messages', thisIndex, 'isSelected'],
          true,
        ],
      })
    )
  }

  /**
   * Should the node card for an element possibly within a chat message be expanded?
   */
  public static shouldExpand(card: HTMLElement, nodeType: NodeType): boolean {
    const types: NodeType[] = [
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

    return (
      types.includes(nodeType) &&
      closestGlobally(card, 'stencila-chat-message[message-role="Model"]') !==
        null
    )
  }

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

  private renderSystemMessage(style: string) {
    return html`
      <div class="${style} my-3 p-3 bg-indigo-100 rounded">
        <slot name="content"></slot>
      </div>
    `
  }

  private renderUserMessage(style: string) {
    return html`
      <div class="${style} flex justify-end">
        <div class="my-3 p-3 bg-blue-50 rounded w-content">
          <slot name="content"></slot>
          <slot name="files"></slot>
        </div>
      </div>
    `
  }

  private renderModelMessage(style: string) {
    return html`<div class="${style} my-3 p-3">
      <slot name="author" class="text-blue-900"></slot>
      ${this.executionStatus === 'Running'
        ? this.renderRunningIndicator()
        : html`
            <slot name="content"></slot>
            ${this.isWithin('ChatMessageGroup')
              ? this.renderSelectButton()
              : ''}
          `}
    </div>`
  }

  private renderRunningIndicator() {
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

  private renderSelectButton() {
    return html`
      <div class="flex justify-center w-full">
        <sl-tooltip content="Select this response">
          <stencila-ui-icon-button
            name=${this.isSelected ? 'checkCircleFill' : 'checkCircle'}
            ?disabled=${this.isSelected}
            @click=${this.onSelected}
          ></stencila-ui-icon-button>
        </sl-tooltip>
      </div>
    `
  }
}
