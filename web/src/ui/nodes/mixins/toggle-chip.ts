import { consume } from '@lit/context'
import { NodeType } from '@stencila/types'
import { apply } from '@twind/core'
import { PropertyValueMap, html } from 'lit'
import { state, property } from 'lit/decorators'

import { DocumentContext, documentContext } from '../../document/context'
import { IconName } from '../../icons/icon'
import { nodeUi } from '../icons-and-colours'

import { UIBaseClass } from './ui-base-class'

export declare class ChipToggleInterface {
  protected documentContext: DocumentContext
  protected renderChip: (icon: IconName, colours: NodeColours) => void
  protected toggle: boolean
  protected toggleChipPosition: string
  protected toggleChip: () => void
  protected dispatchToggleEvent: () => void
  public openCard: () => void
  public closeCard: () => void
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type Constructor<T> = new (...args: any[]) => T
type NodeColours = Pick<
  ReturnType<typeof nodeUi>,
  'borderColour' | 'colour' | 'textColour'
>

const NON_CARD_NODES: NodeType[] = [
  'Article',
  'ListItem',
  'TableCell',
  'TableRow',
  'Text',
  'SuggestionBlock',
]

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

    private static Y_OFFSET_INCREMENT_VALUE: number = 5

    private static MAX_INCREMENTS: number = 4

    private calculateChipOffset() {
      let offset: number = 0
      if (
        this.ancestors &&
        this.depth > 1 &&
        this.constructor.name !== 'UIInlineOnDemand' // exclude 'inline' chips
      ) {
        const ancestors = (this.ancestors.split('.') as NodeType[]) ?? []
        const maxOffset =
          ToggleMixin.Y_OFFSET_INCREMENT_VALUE * ToggleMixin.MAX_INCREMENTS
        ancestors.forEach((node) => {
          if (offset >= maxOffset) {
            return
          }
          if (NON_CARD_NODES.indexOf(node) === -1) {
            offset += ToggleMixin.Y_OFFSET_INCREMENT_VALUE
          }
        })
      }
      return offset
    }

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

    protected renderChip(icon: IconName, colours: NodeColours) {
      const { colour, borderColour, textColour } = colours

      const yOffset = this.calculateChipOffset()

      const styles = apply([
        this.docViewContext.nodeChipState === 'hidden' && 'pointer-events-none',
        !this.toggle &&
          this.docViewContext.nodeChipState !== 'hidden' &&
          'group-hover:opacity-100',
        this.docViewContext.nodeChipState === 'show-all' || this.toggle
          ? 'opacity-100'
          : 'opacity-0',
        'hover:z-50',
        'h-8',
        'flex items-center',
        'transition duration-200',
        'leading-none',
        'px-2 py-1.5',
        `bg-[${colour}]`,
        `border rounded-md border-[${borderColour}]`,
        'cursor-pointer',
        `fill-black text-black`,
        `hover:bg-[${borderColour}] hover:border-[${colour}]`,
        'absolute',
        `top-[${!this.toggle ? yOffset : 0}px]`,
      ])

      return html`
        <div class=${`chip -ml-[40px] ${this.toggleChipPosition}`}>
          <div class=${`${styles}`} @click=${this.toggleChip}>
            <stencila-ui-icon
              name=${this.toggle ? 'chevronDown' : icon}
              class="text-base text-[${textColour}]"
            ></stencila-ui-icon>
          </div>
        </div>
      `
    }

    protected override update(
      // eslint-disable-next-line @typescript-eslint/no-explicit-any
      changedProperties: PropertyValueMap<any> | Map<PropertyKey, unknown>
    ): void {
      super.update(changedProperties)

      if (changedProperties.has('docViewContext')) {
        if (this.docViewContext.nodeChipState === 'hidden' && this.toggle) {
          // collapse open container if `nodeChipState` changes to 'hidden'
          this.toggleChip()
        } else if (
          this.docViewContext.nodeChipState === 'expand-all' &&
          !this.toggle
        ) {
          // expand container `nodeChipState` changes to 'expand-all'
          this.toggleChip()
        }
      }
    }
  }

  return ToggleMixin as unknown as Constructor<ChipToggleInterface> & T
}
