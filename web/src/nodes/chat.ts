import { MutationController } from '@lit-labs/observers/mutation-controller'
import SlCarousel from '@shoelace-style/shoelace/dist/components/carousel/carousel'
import SlCarouselItem from '@shoelace-style/shoelace/dist/components/carousel-item/carousel-item'
import SlSplitPanel from '@shoelace-style/shoelace/dist/components/split-panel/split-panel'
import { apply } from '@twind/core'
import { css, html, PropertyValues } from 'lit'
import { customElement, query, state } from 'lit/decorators'

import { withTwind } from '../twind'
import { nodeUi } from '../ui/nodes/icons-and-colours'

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
  @query('stencila-prompt-block')
  prompt!: PromptBlock

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
