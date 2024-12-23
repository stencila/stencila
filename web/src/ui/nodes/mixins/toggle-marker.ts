import { consume } from '@lit/context'
import { InlineTypeList } from '@stencila/types'
import { apply } from '@twind/core'
import { html } from 'lit'
import { state, property } from 'lit/decorators'

import { ChatMessage } from '../../../nodes/chat-message'
import { closestGlobally } from '../../../utilities/closestGlobally'
import { getModeParam } from '../../../utilities/getModeParam'
import { DocumentContext, documentContext } from '../../document/context'
import { UIBaseCard } from '../cards/base-card'
import { nodeUi } from '../icons-and-colours'

export declare class MarkerToggleInterface {
  protected documentContext: DocumentContext
  protected renderMarker: () => void
  protected toggle: boolean
  protected noVisibleContent: boolean
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
    @consume({ context: documentContext, subscribe: true })
    @state()
    protected docViewContext: DocumentContext

    @state()
    protected toggle: boolean = false

    /**
     * the depth of the current `Node`
     */
    @property({ type: Number })
    depth: number

    /**
     * the string of ancestors for the `Node`
     */
    @property({ type: String })
    ancestors: string

    /**
     * Boolean switch property, to handle nodes with empty content/output
     */
    @property({ type: Boolean })
    protected noVisibleContent: boolean = false

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

    /**
     * Returns a boolean value signaling whether to the
     * card should expand upon render
     */
    protected expandByDefault() {
      const testMode = getModeParam(window)
      if ((testMode && testMode === 'test-expand-all') || this.isRootNode) {
        // set node cards in 'test-expand-all' mode to expand by default
        return true
      } else {
        // If part of a model chat message and included in the list of
        // chat messages auto expanding node types
        return (
          closestGlobally(
            this,
            'stencila-chat-message[message-role="Model"]'
          ) !== null &&
          ChatMessage.DEFAULT_EXPANDED_NODE_CARDS.includes(this.type)
        )
      }
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

    override connectedCallback(): void {
      super.connectedCallback()
      // set node cards in 'test-expand-all' mode to expand by default
      // for regression snapshot testing
      if (this.expandByDefault()) {
        this.toggle = true
      }
    }

    protected renderMarker() {
      const nodeDisplay = InlineTypeList.includes(this.type)
        ? 'inline'
        : 'block'
      let offset = BASE_OFFSET
      if (nodeDisplay === 'block') {
        if (this.noVisibleContent) {
          offset += HORIZ_INSET_PIXELS
        } else if (this.depth > 1 && !this.noVisibleContent) {
          offset -= HORIZ_INSET_PIXELS * (this.depth - 1)
        }
      }

      const { borderColour, icon } = nodeUi(this.type)

      const markerStateClasses =
        this.toggle || this.docViewContext.nodeMarkerState === 'hidden'
          ? 'opacity-0 pointer-events-none'
          : this.docViewContext.nodeMarkerState === 'hover-only'
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
          <div class=${styles}>
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
            </div>
          </div>
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
            @click=${() => (this.toggle = false)}
          ></stencila-ui-icon-button>
        </div>
      `
    }
  }

  return ToggleMixin as unknown as Constructor<MarkerToggleInterface> & T
}
