import { consume } from '@lit/context'
import { InlineTypeList } from '@stencila/types'
import { apply } from '@twind/core'
import { html } from 'lit'
import { state, property } from 'lit/decorators'

import { getModeParam } from '../../../utilities/getModeParam'
import { DocumentContext, documentContext } from '../../document/context'
import { nodeUi } from '../icons-and-colours'

import { UIBaseClass } from './ui-base-class'

import '../chip'

export declare class ChipToggleInterface {
  protected documentContext: DocumentContext
  protected renderChip: () => void
  protected toggle: boolean
  protected noVisibleContent: boolean
  protected toggleChipPosition: string
  protected toggleChip: () => void
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

const HORIZ_INSET_PIXELS = 5

/**
 * A Mixin that provides a "chip" to allow for a card to have its visibility
 * toggled on and off.
 */
export const ToggleChipMixin = <T extends Constructor<UIBaseClass>>(
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
    @state()
    protected noVisibleContent: boolean = false

    /**
     * Used to allow clients to override css classes (tailwind) to change the
     * positioning of the chip.
     */
    protected toggleChipPosition: string = ''

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

    protected toggleChip() {
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

    protected renderChip() {
      const nodeDisplay = InlineTypeList.includes(this.type)
        ? 'inline'
        : 'block'

      let offset = 60
      if (nodeDisplay === 'block' && this.depth > 1) {
        offset -= HORIZ_INSET_PIXELS * (this.depth - 1)
      } else if (this.noVisibleContent) {
        offset += HORIZ_INSET_PIXELS
      }

      const { borderColour, icon } = nodeUi(this.type)

      const chipStateClasses = this.toggle
        ? 'opacity-100'
        : this.docViewContext.nodeChipState === 'hidden'
          ? 'opacity-0 pointer-events-none'
          : this.docViewContext.nodeChipState === 'hover-only'
            ? 'opacity-0 group-hover:opacity-100'
            : 'opacity-100'

      const styles = apply([
        chipStateClasses,
        'absolute top-0',
        'h-full',
        'transition-all duration-200 ease-in-out',
        'hover:cursor-pointer hover:z-50',
        `-left-[${offset}px]`,
        this.toggleChipPosition,
      ])

      const baseMarkerStyles = apply([
        'border-l border-black/10 rounded',
        `bg-[${borderColour}]`,
      ])

      return html`
        <div class=${`chip h-full ${nodeDisplay === 'block' && ''}`}>
          <div class=${styles}>
            ${nodeDisplay === 'block'
              ? html`
                  <div
                    @click=${this.toggleChip}
                    class="relative top-0 left-0 w-2 h-full ${baseMarkerStyles}"
                  ></div>
                `
              : ''}
            <div
              @click=${this.toggleChip}
              class="flex justify-center items-center absolute top-0 left-0 w-6 h-6 text-sm ${baseMarkerStyles} rounded-bl-none"
            >
              <stencila-ui-icon class="text-xs" name=${icon}>
              </stencila-ui-icon>
            </div>
          </div>
        </div>
      `
    }

    override connectedCallback(): void {
      super.connectedCallback()
      const testMode = getModeParam(window)
      if (testMode && testMode === 'test-expand-all') {
        this.toggle = true
      }
    }
  }

  return ToggleMixin as unknown as Constructor<ChipToggleInterface> & T
}
