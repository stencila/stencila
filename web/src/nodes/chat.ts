import { provide } from '@lit/context'
import { MutationController } from '@lit-labs/observers/mutation-controller'
import SlCarousel from '@shoelace-style/shoelace/dist/components/carousel/carousel'
import SlCarouselItem from '@shoelace-style/shoelace/dist/components/carousel-item/carousel-item'
import SlSplitPanel from '@shoelace-style/shoelace/dist/components/split-panel/split-panel'
import { apply } from '@twind/core'
import { css, html, PropertyValues } from 'lit'
import { customElement, state } from 'lit/decorators'

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
   * A mutation controller used to expand the suggestions
   * slider open when a suggestion is added
   *
   * @see onSuggestionSlotChange
   */
  // @ts-expect-error is never read
  private suggestionsMutationController: MutationController

  /**
   * The number of suggestions
   *
   * Currently only used for changing the position of
   * the split panel when a suggestion is first added.
   *
   * In the future, an indicator could be provided to the
   * user to show the number of suggestions, when that panel is closed.
   *
   * @see onSuggestionSlotChange
   */
  @state()
  private suggestionsCount: number = 0

  /**
   * The position of the split panel
   *
   * Used for changing the position of the split panel when a suggestion
   * is first added and keeping track of position selected by user.
   *
   * Not a @state to avoid reactive loop.
   */
  private splitPosition: number = 100

  /**
   * When user changes the split position record the position so that
   * it can be used on the next `render()`.
   */
  onSplitPositionChange(event: Event) {
    const panel = event.target as SlSplitPanel
    if (panel.position) {
      this.splitPosition = panel.position
    }
  }

  /**
   * On a change to the prompt slot update the instruction
   * type of the chat
   */
  private onPromptSlotChange({ target: slot }: Event) {
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
  private onContentSlotChange({ target: slot }: Event) {
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

  /**
   * On a change to the suggestions slot initialize the
   * mutation controller to open the slider.
   *
   * Only applied for top level chats.
   */
  private onSuggestionsSlotChange({ target: slot }: Event) {
    if (this.depth > 0) {
      return
    }

    const suggestionsElem = (slot as HTMLSlotElement).assignedElements()[0]
    if (!suggestionsElem) {
      return
    }

    const update = () => {
      const count = suggestionsElem.querySelectorAll(
        'stencila-suggestion-block'
      ).length

      // If the first suggestion has been added and the suggestions panel
      // is currently hidden then make the split 50%
      if (this.suggestionsCount === 0 && count > 0 && this.splitPosition > 95) {
        this.splitPosition = 50
      }

      if (count != this.suggestionsCount) {
        this.suggestionsCount = count
      }
    }
    update()

    this.suggestionsMutationController = new MutationController(this, {
      target: suggestionsElem,
      config: { childList: true, subtree: true },
      callback: update,
    })
  }

  override render() {
    return this.depth == 0 ? this.renderFullscreen() : this.renderCard()
  }

  private renderFullscreen() {
    const { borderColour, colour, textColour } = nodeUi('Chat')

    const headerClasses = apply([
      `bg-[${colour}] border-b border-[${borderColour}]`,
      'px-3 py-1',
      `font-sans font-semibold text-sm text-[${textColour}]`,
    ])

    const footerClasses = apply([
      `bg-[${colour}] border-t border-[${borderColour}]`,
    ])

    return html`
      <div class="h-screen w-screen flex flex-col">
        <div class=${headerClasses}>Chat</div>

        <div class="flex-grow overflow-y-hidden">
          <sl-split-panel
            class="h-full"
            position=${this.splitPosition}
            @sl-reposition=${this.onSplitPositionChange}
          >
            <div slot="start" class="px-3 overflow-y-auto">
              <slot
                name="content"
                @slotchange=${this.onContentSlotChange}
              ></slot>
            </div>
            <div slot="end" class="px-1 py-2">
              <slot
                name="suggestions"
                @slotchange=${this.onSuggestionsSlotChange}
              ></slot>
            </div>
          </sl-split-panel>
        </div>

        <div class=${footerClasses}>
          <div class="p-1">
            <div class="max-w-prose mx-auto">
              <stencila-ui-chat-message-inputs
                type="Chat"
                node-id=${this.id}
              ></stencila-ui-chat-message-inputs>
            </div>
          </div>

          <stencila-ui-node-execution-messages type="Chat">
            <slot name="execution-messages"></slot>
          </stencila-ui-node-execution-messages>

          <slot name="model-parameters"></slot>

          <slot name="prompt"></slot>
        </div>
      </div>
    `
  }

  private renderCard() {
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
    </stencila-ui-block-on-demand>`
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
