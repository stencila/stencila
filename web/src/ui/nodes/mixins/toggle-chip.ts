import { consume } from '@lit/context'
import { apply } from '@twind/core'
import { PropertyValueMap, html } from 'lit'
import { state } from 'lit/decorators'

import {
  DocPreviewContext,
  documentPreviewContext,
} from '../../../contexts/preview-context'
import { nodeUi } from '../icons-and-colours'

import { UIBaseClass } from './ui-base-class'

export declare class ChipToggleInterface {
  protected docViewContext: DocPreviewContext
  protected renderChip: (icons: [string, string], colours: NodeColours) => void
  protected toggle: boolean
  protected toggleChipPosition: string
  protected toggleChip: () => void
  protected dispatchToggleEvent: () => void
  public openCard: () => void
  public closeCard: () => void
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type Constructor<T> = new (...args: any[]) => T
type NodeColours = Pick<ReturnType<typeof nodeUi>, 'borderColour' | 'colour'>

/**
 * A Mixin that provides a "chip" to allow for a card to have its visibility
 * toggled on and off.
 */
export const ToggleChipMixin = <T extends Constructor<UIBaseClass>>(
  superClass: T
) => {
  class ToggleMixin extends superClass {
    @consume({ context: documentPreviewContext, subscribe: true })
    @state()
    protected docViewContext: DocPreviewContext

    @state()
    protected toggle: boolean = false

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

    protected renderChip(icons: [string, string], colours: NodeColours) {
      const { colour, borderColour } = colours
      const [library, icon] = icons

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
        'top-0',
      ])

      return html`
        <div class=${`chip -ml-[40px] ${this.toggleChipPosition}`}>
          <div class=${`${styles}`} @click=${this.toggleChip}>
            <sl-icon
              library=${this.toggle ? 'default' : library}
              name=${this.toggle ? 'chevron-down' : icon}
              class="text-base"
            ></sl-icon>
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
