import { provide } from '@lit/context'
import { MutationController } from '@lit-labs/observers/mutation-controller'
import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'
import { ChatContext, chatContext } from '../ui/nodes/chat-context'
import { nodeUi } from '../ui/nodes/icons-and-colours'

import { ChatMessage } from './chat-message'
import { Executable } from './executable'
import { PromptBlock } from './prompt-block'

/**
 * Web component representing a Stencila `Chat` node
 *
 * A `Chat` node is similar to `Article` and `Prompt` nodes in that they
 * are usually (but not necessarily) the root node of a document.
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/works/chat.md
 */
@customElement('stencila-chat')
@withTwind()
export class StencilaChat extends Executable {
  /**
   * The chat context, used to update the UI of nodes within
   * the chat according to its properties.
   */
  @provide({ context: chatContext })
  private chatContext?: ChatContext = {
    instructionType: undefined,
    source: undefined,
  }

  /**
   * A mutation controller used to update the instruction type of the chat
   *
   * @see onPromptSlotChange
   */
  // @ts-expect-error is never read
  private promptMutationController: MutationController

  /**
   * A mutation controller used to scroll to the latest message
   *
   * @see onContentSlotChange
   */
  // @ts-expect-error is never read
  private contentMutationController: MutationController

  /**
   * On a change to the prompt slot update the instruction
   * type of the chat
   */
  onPromptSlotChange({ target: slot }: Event) {
    const promptElem = (slot as HTMLSlotElement).assignedElements()[0]
    if (!(promptElem instanceof PromptBlock)) {
      return
    }

    this.chatContext.instructionType = promptElem.instructionType

    this.promptMutationController = new MutationController(this, {
      target: promptElem,
      config: {
        attributes: true,
      },
      callback: (mutations) => {
        for (const mutation of mutations) {
          if (
            mutation.target instanceof PromptBlock &&
            mutation.attributeName === 'instruction-type'
          ) {
            this.chatContext.instructionType = mutation.target.instructionType
          }
        }
      },
    })
  }

  /**
   * On a change to the content slot initialize the
   * mutation controller to scroll to the most recent message
   *
   * Note that an empty chat starts with no `content` slot so
   * this is why we use a slot change event to observe it when
   * that slot is populated.
   *
   * Only applied for top level chats.
   */
  onContentSlotChange({ target: slot }: Event) {
    if (this.depth > 0) {
      return
    }

    const contentElem = (slot as HTMLSlotElement).assignedElements()[0]
    if (!contentElem) {
      return
    }

    this.contentMutationController = new MutationController(this, {
      target: contentElem,
      config: {
        attributes: true,
        childList: true,
        subtree: true,
      },
      callback: (mutations) => {
        // Find the first chat message that we may need to scroll to:
        // was added, or had content added to it.
        let elem: ChatMessage | undefined
        for (const mutation of mutations) {
          if (mutation.target instanceof ChatMessage) {
            elem = mutation.target
            break
          } else if (mutation.target.parentElement instanceof ChatMessage) {
            elem = mutation.target.parentElement
            break
          } else if (mutation.addedNodes[0] instanceof ChatMessage) {
            elem = mutation.addedNodes[0]
            break
          }
        }

        if (elem) {
          requestAnimationFrame(() => {
            const elemRect = elem.getBoundingClientRect()
            const top = elemRect.top + window.scrollY

            if (elemRect.height > window.innerHeight) {
              // For tall elements, scroll to their top
              window.scrollTo({
                top,
                behavior: 'smooth',
              })
            } else {
              // For shorter elements, scroll to their bottom + padding
              // (for fixed message input box)
              window.scrollTo({
                top: top + elemRect.height + 150,
                behavior: 'smooth',
              })
            }
          })
        }
      },
    })
  }

  override render() {
    return html`<stencila-ui-block-on-demand
      type="Chat"
      node-id=${this.id}
      depth=${this.depth}
    >
      <div slot="header-right" class="flex">
        <stencila-ui-node-execution-commands
          type="Chat"
          node-id=${this.id}
          depth=${this.depth}
        >
        </stencila-ui-node-execution-commands>
      </div>

      <div slot="body">
        <stencila-ui-node-execution-details
          type="Chat"
          node-id=${this.id}
          mode=${this.executionMode}
          bounds=${this.executionBounds}
          .tags=${this.executionTags}
          status=${this.executionStatus}
          required=${this.executionRequired}
          count=${this.executionCount}
          ended=${this.executionEnded}
          duration=${this.executionDuration}
        >
        </stencila-ui-node-execution-details>

        <stencila-ui-node-execution-messages type="Chat">
          <slot name="execution-messages"></slot>
        </stencila-ui-node-execution-messages>

        <slot name="model-parameters"></slot>

        <slot name="prompt"></slot>
      </div>

      ${this.renderContent()}
    </stencila-ui-block-on-demand>`
  }

  private renderContent() {
    return this.depth === 0
      ? html`<div slot="content" class="pb-20">
          <slot name="content" @slotchange=${this.onContentSlotChange}></slot>
          ${this.renderInputPanel()}
        </div>`
      : ''
  }

  private renderInputPanel() {
    const { borderColour, colour } = nodeUi('Chat')

    return html` <div
      class="fixed bottom-0 left-0 z-10 w-full
      border border-t-[${borderColour}]
      bg-[${colour}] opacity-95
      p-1"
    >
      <div class="max-w-prose mx-auto">
        <stencila-ui-chat-message-inputs
          type="Chat"
          node-id=${this.id}
        ></stencila-ui-chat-message-inputs>
      </div>
    </div>`
  }
}
