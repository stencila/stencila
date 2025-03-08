import { MutationController } from '@lit-labs/observers/mutation-controller'
import SlCarousel from '@shoelace-style/shoelace/dist/components/carousel/carousel'
import SlCarouselItem from '@shoelace-style/shoelace/dist/components/carousel-item/carousel-item'
import { apply } from '@twind/core'
import { css, html, PropertyValues } from 'lit'
import { customElement, property } from 'lit/decorators'

import { archiveNode } from '../clients/commands'
import { withTwind } from '../twind'
import { nodeUi } from '../ui/nodes/icons-and-colours'
import '../ui/nodes/chat/chat-message-inputs'
import '../ui/nodes/nodes-selected'

import { ChatMessage } from './chat-message'
import { ChatMessageGroup } from './chat-message-group'
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
export class Chat extends Executable {
  @property({ attribute: 'target-nodes', type: Array })
  targetNodes?: string[]

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
   * On a change to the content slot initialize the
   * mutation controller to scroll to the most recent message
   *
   * Note that an empty chat starts with no `content` slot so
   * this is why we use a slot change event to observe it when
   * that slot is populated.
   *
   * Only applied for top level chats.
   */
  private onContentSlotChange({ target: slot }: Event) {
    if (this.depth > 0) {
      return
    }

    const contentElem = (slot as HTMLSlotElement).assignedElements()[0]
    if (!contentElem) {
      return
    }

    // Get the visible height of the content element
    const visibleHeight = () => {
      const rect = contentElem.getBoundingClientRect()
      const viewportHeight = window.innerHeight

      const visibleTop = Math.max(rect.top, 0)
      const visibleBottom = Math.min(rect.bottom, viewportHeight)

      return Math.max(0, visibleBottom - visibleTop)
    }

    // Scroll to the appropriate place based on new messages
    const scroll = (mutations: MutationRecord[]) => {
      // Get the first and last messages added
      let first: ChatMessage | ChatMessageGroup | undefined
      let last: ChatMessage | ChatMessageGroup | undefined
      for (const mutation of mutations) {
        if (
          mutation.target instanceof ChatMessage &&
          mutation.type === 'attributes' &&
          mutation.attributeName === 'is-selected'
        ) {
          // if mutation is chat group selection change,
          // scroll to top of chat message group.
          if (mutation.target.isSelected) {
            const targetEl =
              (mutation.target.closest(
                'stencila-chat-message-group'
              ) as ChatMessageGroup) ?? mutation.target
            targetEl.scrollIntoView({
              block: 'start',
              behavior: 'smooth',
            })
            return
          }
        } else {
          let elem
          if (
            mutation.target instanceof ChatMessage ||
            mutation.target instanceof ChatMessageGroup
          ) {
            elem = mutation.target
          } else if (
            mutation.target.parentElement instanceof ChatMessage ||
            mutation.target.parentElement instanceof ChatMessageGroup
          ) {
            elem = mutation.target.parentElement
          } else if (
            mutation.addedNodes[0] instanceof ChatMessage ||
            mutation.addedNodes[0] instanceof ChatMessageGroup
          ) {
            elem = mutation.addedNodes[0]
          }

          if (!first) {
            first = elem
            last = elem
          } else {
            last = elem
          }
        }
      }

      if (first) {
        requestAnimationFrame(() => {
          // After fully rendered, get combined height of added content
          const height =
            last.getBoundingClientRect().bottom -
            first.getBoundingClientRect().top

          // If taller than visible content, scroll to top of first, otherwise to end of last
          if (height > visibleHeight()) {
            first.scrollIntoView({
              block: 'start',
              behavior: 'smooth',
            })
          } else {
            last.scrollIntoView({
              block: 'end',
              behavior: 'smooth',
            })
          }
        })
      }
    }

    this.contentMutationController = new MutationController(this, {
      target: contentElem,
      config: {
        attributes: true,
        childList: true,
        subtree: true,
      },
      callback: scroll,
    })
  }

  /**
   * On message input, forward the input value to the prompt component
   * for potential use as an implied query for prompt target
   */
  private onMessageInput({ detail: value }: CustomEvent) {
    const prompt = this.querySelector('stencila-prompt-block') as PromptBlock
    prompt.onQueryImplied(value)
  }

  /**
   * Generate placeholder test for the messages input based on
   * the instruction type and the number of existing messages.
   */
  private getPlaceholder(): string {
    const prompt = this.querySelector(
      'stencila-prompt-block'
    ) as PromptBlock | null

    if (!prompt) {
      return ''
    }

    const messages = this.querySelectorAll('stencila-chat-message')
    if (messages && messages.length > 0) {
      return 'What would you like to improve?'
    }

    switch (prompt.instructionType) {
      case 'Create':
        return 'What would you like to create?'
      case 'Discuss':
        return 'What would you like to discuss?'
      case 'Edit':
        return 'How would you like to edit this?'
      case 'Fix':
      case 'Describe':
        return ''
    }
  }

  override render() {
    return this.depth == 0 ? this.renderFullscreen() : html``
  }

  private renderFullscreen() {
    const { borderColour, colour, textColour } = nodeUi('Chat')

    const headerClasses = apply(
      'flex flex-row justify-between items-center',
      `bg-[${colour}] border-b border-[${borderColour}]`,
      'px-3 pt-2 pb-1',
      `font-sans font-semibold text-sm text-[${textColour}]`
    )

    return html`
      <div class="h-screen w-screen flex flex-col">
        <div class=${headerClasses}>
          <div>Chat</div>

          <div class="flex flex-row items-center gap-3">
            <sl-tooltip content="Archive chat">
              <stencila-ui-icon-button
                name="archive"
                @click=${() => this.dispatchEvent(archiveNode('Chat', this.id))}
              >
                Archive
              </stencila-ui-icon-button>
            </sl-tooltip>
          </div>
        </div>

        <div class="flex-grow overflow-y-hidden">
          <div class="h-full overflow-y-hidden flex flex-col">
            <div class="flex-grow overflow-y-hidden">
              <div
                class="relative h-full overflow-auto"
                id="chat-scroll-container"
              >
                <div class="px-3 pb-6 min-w-[50ch] max-w-[80ch] mx-auto">
                  <slot
                    name="content"
                    @slotchange=${this.onContentSlotChange}
                  ></slot>
                  <stencila-ui-nodes-selected
                    type="Chat"
                    node-id=${this.id}
                  ></stencila-ui-nodes-selected>
                </div>
              </div>
            </div>

            <div
              class="bg-[${colour}] border-t border-[${borderColour}] px-3 py-2"
            >
              <div class="pb-2">
                <slot name="prompt"></slot>
              </div>

              <stencila-ui-chat-message-inputs
                type="Chat"
                node-id=${this.id}
                placeholder=${this.getPlaceholder()}
                @stencila-message-input=${this.onMessageInput}
              >
                <slot name="model-parameters" slot="model-parameters"></slot>
              </stencila-ui-chat-message-inputs>

              <stencila-ui-node-execution-messages type="Chat">
                <slot name="execution-messages"></slot>
              </stencila-ui-node-execution-messages>
            </div>
          </div>
        </div>
      </div>
    `
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

  /**
   * Override to go to a new suggestion when it has been appended
   */
  override appendChild<T extends Node>(node: T): T {
    const appended = super.appendChild(node)

    const index =
      this.querySelectorAll('stencila-chat-suggestions-item').length - 1

    requestAnimationFrame(() => this.goToSlide(index, 'smooth'))

    return appended
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
        overflow-y: auto;
      }
    `,
  ]
}
