import { provide } from '@lit/context'
import { MutationController } from '@lit-labs/observers/mutation-controller'
import SlCarousel from '@shoelace-style/shoelace/dist/components/carousel/carousel'
import SlCarouselItem from '@shoelace-style/shoelace/dist/components/carousel-item/carousel-item'
import { css, html, PropertyValues } from 'lit'
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
      <div slot="header-right">
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

      ${this.depth === 0
        ? html`<div slot="content" class="flex flex-col">
            ${this.renderContent()}
          </div>`
        : ''}
    </stencila-ui-block-on-demand>`
  }

  private renderContent() {
    let content = html`<slot
      name="content"
      @slotchange=${this.onContentSlotChange}
    ></slot>`

    const suggestions = this.querySelector(
      '[slot=suggestions]'
    ) as SlCarousel | null
    if (suggestions) {
      // TODO: `h-[75vh]` is temporary fix related to having a fixed footer; probably better to add a footer slot
      // to the card and making the whole card `h-screen`
      content = html`<sl-split-panel class="h-[70vh] pb-6">
        <div slot="start" class="px-3 overflow-scroll">${content}</div>
        <div slot="end" class="px-1">
          <slot name="suggestions"></slot>
        </div>
      </sl-split-panel>`
    }

    return html`${content} ${this.renderInputPanel()}`
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

@customElement('stencila-chat-suggestions')
@withTwind()
// @ts-expect-error TS does not like override
export class ChatSuggestions extends SlCarousel {
  static override styles = [
    SlCarousel.styles,
    css`
      :host {
        --aspect-ratio: none;
        height: 100%;
      }

      :host::part(navigation-button) {
        padding: 0;
      }
    `,
  ]

  /**
   * Override needed to reliably set attributes that affect
   * rendering (multiple alternative approaches to this failed)
   */
  protected override update(changedProperties: PropertyValues): void {
    this.navigation = true
    this.pagination = true
    this.mouseDragging = true

    super.update(changedProperties)
  }

  /**
   * Override needed to function with <stencila-chat-suggestions-item> children
   */
  // @ts-expect-error TS does not like this override of a private method
  private override isCarouselItem(node: Node): node is ChatSuggestionsItem {
    return (
      node instanceof Element &&
      node.tagName.toLowerCase() === 'stencila-chat-suggestions-item'
    )
  }
}

@customElement('stencila-chat-suggestions-item')
@withTwind()
export class ChatSuggestionsItem extends SlCarouselItem {
  static override styles = [
    SlCarouselItem.styles,
    css`
      :host {
        height: 100%;
      }
    `,
  ]
}
