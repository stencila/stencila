import { consume } from '@lit/context'
import { InlineTypeList } from '@stencila/types'
import { apply } from '@twind/core'
import { html } from 'lit'
import { state, property } from 'lit/decorators'

import { Article } from '../../../nodes/article'
import { ChatMessage } from '../../../nodes/chat-message'
import { Excerpt } from '../../../nodes/excerpt'
import { SuggestionBlock } from '../../../nodes/suggestion-block'
import { getModeParam } from '../../../utilities/getModeParam'
import { DocumentContext, documentContext } from '../../document/context'
import { UIBaseCard } from '../cards/base-card'
import { nodeUi } from '../icons-and-colours'

export declare class MarkerToggleInterface {
  protected documentContext: DocumentContext
  protected renderMarker: () => void
  protected toggle: boolean
  protected noContent: boolean
  protected toggleMarkerPosition: string
  protected toggleMarker: () => void
  protected dispatchToggleEvent: () => void
  public openCard: () => void
  public closeCard: () => void
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type Constructor<T> = new (...args: any[]) => T
export type NodeColours = Pick<
  ReturnType<typeof nodeUi>,
  'borderColour' | 'colour' | 'textColour'
>

// Nested
const HORIZ_INSET_PIXELS = 5
// The base offset in pixels for the node marker
const BASE_OFFSET = 60
// The number in pixels to remove from the offset for smaller screens
const SMALL_DEVICE_OFFSET_MODIFIER = 10

/**
 * A Mixin that provides a "marker" with a vertical bar, to allow for a card to have its visibility
 * toggled on and off.
 */
export const ToggleMarkerMixin = <T extends Constructor<UIBaseCard>>(
  superClass: T
) => {
  abstract class ToggleMixin extends superClass {
    /**
     * Boolean switch property, to handle nodes with empty content/output
     */
    @property({ attribute: 'no-content', type: Boolean })
    protected noContent: boolean = false

    @consume({ context: documentContext, subscribe: true })
    @state()
    protected docViewContext: DocumentContext

    @state()
    protected toggle: boolean = false

    /**
     * Used to allow clients to override css classes (tailwind) to change the
     * positioning of the marker.
     */
    protected toggleMarkerPosition: string = ''

    // ----------------------
    // public methods for allow opening / closing the card externally.
    public openCard() {
      this.toggle = true
      this.dispatchToggleEvent()
    }

    public closeCard() {
      this.toggle = false
      this.dispatchToggleEvent()
    }
    // ---------------------

    protected toggleMarker() {
      this.toggle = !this.toggle
      this.dispatchToggleEvent()
    }

    protected dispatchToggleEvent() {
      this.shadowRoot.dispatchEvent(
        new CustomEvent(`toggle-${this.nodeId}`, {
          bubbles: true,
          composed: true,
          detail: { cardOpen: this.toggle, nodeId: this.nodeId },
        })
      )
    }

    /**
     * Whether the node card should be initially expanded
     */
    protected isInitiallyExpanded() {
      // Expand if the root node
      if (this.depth === 0 && this.hasRoot) {
        return true
      }

      // Expand if in 'test-expand-all' mode for snapshot tests
      const testMode = getModeParam(window)
      if (testMode && testMode === 'test-expand-all') {
        return true
      }

      // Expand certain nodes types in certain contexts
      if (
        Article.shouldExpand(this, this.type) ||
        Excerpt.shouldExpand(this, this.type) ||
        ChatMessage.shouldExpand(this, this.type) ||
        SuggestionBlock.shouldExpand(this, this.type)
      ) {
        return true
      }

      return false
    }

    override connectedCallback(): void {
      super.connectedCallback()

      this.toggle = this.isInitiallyExpanded()
    }

    protected renderMarker() {
      const nodeDisplay = InlineTypeList.includes(this.type)
        ? 'inline'
        : 'block'

      let offset = BASE_OFFSET
      if (nodeDisplay === 'block') {
        if (this.noContent) {
          offset += HORIZ_INSET_PIXELS
        } else if (this.depth > 1) {
          offset -= HORIZ_INSET_PIXELS * (this.depth - 1)
        }
      }

      const { borderColour, icon } = nodeUi(this.type)

      const defaultState = this.docViewContext?.nodeMarkerState ?? 'hover-only'

      const markerStateClasses =
        this.toggle || defaultState === 'hidden'
          ? 'opacity-0 pointer-events-none'
          : defaultState === 'hover-only'
            ? 'opacity-0 group-hover:opacity-100'
            : 'opacity-100'

      const styles = apply([
        markerStateClasses,
        'absolute top-0',
        'h-full',
        'transition-all duration-200 ease-in-out',
        'hover:cursor-pointer hover:z-50',
        `-left-[${offset - SMALL_DEVICE_OFFSET_MODIFIER}px] sm:-left-[${offset}px]`,
        this.toggleMarkerPosition,
      ])

      const baseMarkerStyles = apply([
        'border-l border-black/10 rounded',
        `bg-[${borderColour}]`,
      ])

      return html`
        <div
          class=${`chip absolute top-0 h-full ${nodeDisplay === 'block' && ''}`}
        >
          <sl-tooltip content="Inspect ${this.type}"
            ><div class=${styles}>
              ${nodeDisplay === 'block'
                ? html`
                    <div
                      @click=${this.toggleMarker}
                      class="relative top-0 left-0 w-2 h-full ${baseMarkerStyles}"
                    ></div>
                  `
                : ''}
              <div
                @click=${this.toggleMarker}
                class="flex justify-center items-center absolute top-0 left-0 w-6 h-6 text-sm ${baseMarkerStyles} rounded-bl-none"
              >
                <stencila-ui-icon class="text-xs" name=${icon}>
                </stencila-ui-icon>
              </div></div
          ></sl-tooltip>
        </div>
      `
    }

    /**
     * Overrides the base card empty `renderClose` method,
     * this allows closing the node card from the header.
     */
    protected override renderClose() {
      const classes = apply(['flex items-center', 'ml-3'])
      return html`
        <div class=${classes}>
          <stencila-ui-icon-button
            name="x"
            @click=${() => this.closeCard()}
          ></stencila-ui-icon-button>
        </div>
      `
    }
  }

  return ToggleMixin as unknown as Constructor<MarkerToggleInterface> & T
}
