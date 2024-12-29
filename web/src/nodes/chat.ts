import { provide } from '@lit/context'
import { MutationController } from '@lit-labs/observers/mutation-controller'
import SlCarousel from '@shoelace-style/shoelace/dist/components/carousel/carousel'
import SlCarouselItem from '@shoelace-style/shoelace/dist/components/carousel-item/carousel-item'
import { css } from '@twind/core'
import { Idiomorph } from 'idiomorph/dist/idiomorph.esm.js'
import { html } from 'lit'
import { customElement } from 'lit/decorators'
import { createRef, ref, Ref } from 'lit/directives/ref'

import { withTwind } from '../twind'
import { ChatContext, chatContext } from '../ui/nodes/chat-context'
import { nodeUi } from '../ui/nodes/icons-and-colours'

import { ChatMessage } from './chat-message'
import { Executable } from './executable'
import { PromptBlock } from './prompt-block'
import { SuggestionBlock } from './suggestion-block'

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
   * A mutation controller used to create a carousel for suggestions
   *
   * @see onSuggestionsSlotChange
   */
  // @ts-expect-error is never read
  private suggestionsMutationController: MutationController

  /**
   * A reference to the element containing the carousel items for each suggestion
   */
  private suggestionsCarousel: Ref<SlCarousel> = createRef()

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

  /**
   * On a change to the suggestions slot, clone each suggestion and
   * place inside a `<sl-carousel-item>`.
   *
   * This is hacky but seems to be the only way to use `<sl-carousel>`
   * because its children must be `<sl-carousel-item>`s.
   */
  onSuggestionsSlotChange({ target: slot }: Event) {
    const suggestionsElem = (slot as HTMLSlotElement).assignedElements()[0]
    if (!suggestionsElem) {
      return
    }

    // Create a carousel item for a suggestion
    const suggestionAsCarouselItem = (
      suggestion: SuggestionBlock
    ): SlCarouselItem => {
      // Clone each suggestion so that when we append it to the carousel item
      // we do not remove it from the light DOM, and thus create more mutation
      // events than necessary when the light DOM is updated.
      const clone = suggestion.cloneNode(true) as SuggestionBlock

      // Clone does not clone id, so do that so that commands work
      clone.id = suggestion.id

      clone.className =
        'h-full w-full min-w-[45ch] max-w-prose mx-auto p-5 overflow-scroll'

      const carouselItem = document.createElement('sl-carousel-item')
      carouselItem.className = 'h-full'
      carouselItem.appendChild(clone)

      return carouselItem
    }

    const suggestionIntoCarouselItem = (
      suggestion: SuggestionBlock,
      item: SlCarouselItem
    ): void => {
      // Use Idiomorph to merge the new state of the suggestion into the carousel item
      Idiomorph.morph(item.firstElementChild, suggestion)
    }

    // Start by adding any existing suggestions
    Array.from(suggestionsElem.children).forEach(
      (suggestion: SuggestionBlock) => {
        this.suggestionsCarousel.value.appendChild(
          suggestionAsCarouselItem(suggestion)
        )
      }
    )

    const reconcile = () => {
      Array.from(suggestionsElem.children).forEach(
        (suggestion: SuggestionBlock, index: number) => {
          const item = this.suggestionsCarousel.value.children[index]
          if (item) {
            suggestionIntoCarouselItem(suggestion, item as SlCarouselItem)
          } else {
            this.suggestionsCarousel.value.appendChild(
              suggestionAsCarouselItem(suggestion)
            )
          }
        }
      )
    }

    this.suggestionsMutationController = new MutationController(this, {
      target: suggestionsElem,
      config: {
        attributes: true,
        childList: true,
        subtree: true,
      },
      callback: () => {
        reconcile()
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

    const suggestions = this.querySelector('[slot=suggestions]')
    if (suggestions) {
      const carouselClass = css`
        & {
          --aspect-ratio: none;
        }
        &::part(navigation-button) {
          padding: 0;
        }
      `

      // TODO: `h-[75vh]` is temporary fix related to having a fixed footer; probably better to add a footer slot
      // to the card and making the whole card `h-screen`
      content = html`<sl-split-panel class="h-[70vh] pb-6">
          <div slot="start" class="px-3 overflow-scroll">${content}</div>

          <div slot="end" class="px-1">
            <sl-carousel
              pagination
              navigation
              mouse-dragging
              class="h-full ${carouselClass}"
              ${ref(this.suggestionsCarousel)}
            >
              <stencila-ui-icon
                name="chevronLeft"
                slot="previous-icon"
              ></stencila-ui-icon>

              <stencila-ui-icon
                name="chevronRight"
                slot="next-icon"
              ></stencila-ui-icon>
            </sl-carousel>
          </div>
        </sl-split-panel>

        <div class="hidden">
          <slot
            name="suggestions"
            @slotchange=${this.onSuggestionsSlotChange}
          ></slot>
        </div>`
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
