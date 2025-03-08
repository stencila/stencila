import { NodeType } from '@stencila/types'
import { apply } from '@twind/core'
import { html, PropertyValues } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../twind'
import { booleanConverter } from '../utilities/booleanConverter'
import { closestGlobally } from '../utilities/closestGlobally'

import { Executable } from './executable'

import '../ui/nodes/chat/chat-message-inputs'

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
    converter: booleanConverter,
  })
  isSelected?: boolean = false

  /**
   * Indicates whether to render the execution messages slot
   */
  private hasExecutionMessages: boolean = false

  private observer: MutationObserver

  /**
   * Should a node card, possibly within a chat message, be expanded?
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
      // Expand these node types in chat content, which includes blocks that
      // are the target of the chat, as well as chat messages,
      // but which excludes nodes in the prompt preview.
      closestGlobally(card, 'stencila-chat > [slot="content"]') !== null
    )
  }

  override connectedCallback(): void {
    super.connectedCallback()

    // set up observer for changes in the execution message slot
    this.observer = new MutationObserver(() => {
      const messages = this.querySelector('[slot="execution-messages"]')
      if (messages && messages.children.length > 0) {
        this.hasExecutionMessages = true
      } else {
        this.hasExecutionMessages = false
      }
    })

    this.observer.observe(this, { childList: true })
  }

  protected override firstUpdated(changedProperties: PropertyValues): void {
    super.firstUpdated(changedProperties)
    // set first message in group as selected by default
    if (this.isWithin('ChatMessageGroup')) {
      this.isSelected = this.parentNode.children[0].isSameNode(this)
    }
  }

  override disconnectedCallback(): void {
    super.disconnectedCallback()
    this.observer.disconnect()
  }

  override render() {
    // These styles are applied here, rather than any container in
    // a chat because in a chat message group the messages within
    // each group need to be limited in width
    const style = apply('min-w-[45ch] max-w-[80ch] mx-auto')

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
        ${this.hasExecutionMessages
          ? html` <div class="bg-gray-100 rounded">
              <slot name="execution-messages"></slot>
            </div>`
          : ''}
        <div class="bg-gray-100 rounded">
          <slot name="execution-messages"></slot>
        </div>
        <slot name="content"></slot>
      </div>
    `
  }

  private renderUserMessage(style: string) {
    return html`
      <div class="${style} flex justify-end">
        ${this.hasExecutionMessages
          ? html`
              <div>
                <slot name="execution-messages"></slot>
              </div>
            `
          : ''}
        <div class="my-3 p-3 bg-blue-50 rounded w-content">
          <slot name="content"></slot>
          <slot name="files"></slot>
        </div>
      </div>
    `
  }

  private renderModelMessage(style: string) {
    const inGroup = this.isWithin('ChatMessageGroup')

    if (!inGroup) {
      return html`
        <div class="${style} my-3">
          <div class="mb-4">
            <slot
              name="author"
              class=${this.executionStatus === 'Running'
                ? 'text-gray-400'
                : 'text-brand-blue'}
            ></slot>
          </div>
          ${this.hasExecutionMessages
            ? html` <div class="bg-gray-100 rounded">
                <slot name="execution-messages"></slot>
              </div>`
            : ''}
          <slot name="content"></slot>
          ${this.executionStatus === 'Running'
            ? this.renderRunningIndicator()
            : ''}
        </div>
      `
    }

    return this.isSelected
      ? html`
          <div class="${style} my-3">
            <div class="bg-gray-100 rounded">
              <slot name="execution-messages"></slot>
            </div>
            <slot name="content"></slot>
            ${this.executionStatus === 'Running'
              ? this.renderRunningIndicator()
              : ''}
          </div>
        `
      : html``
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
}
