import { consume } from '@lit/context'
import { InlineTypeList, NodeType } from '@stencila/types'
import { apply } from '@twind/core'
import { html } from 'lit'
import { state, property } from 'lit/decorators'

import { DocumentContext, documentContext } from '../../document/context'
import { nodeUi } from '../icons-and-colours'

import { UIBaseClass } from './ui-base-class'

import '../chip'

export declare class ChipToggleInterface {
  protected documentContext: DocumentContext
  protected renderChip: (node: NodeType) => void
  protected toggle: boolean
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

    protected renderChip(node: NodeType) {
      const nodeDisplay = InlineTypeList.includes(this.type)
        ? 'inline'
        : 'block'

      const chipStateClasses = this.toggle
        ? 'opacity-100'
        : this.docViewContext.nodeChipState === 'hidden'
          ? 'opacity-0 pointer-events-none'
          : this.docViewContext.nodeChipState === 'hover-only'
            ? 'opacity-0 group-hover:opacity-100'
            : 'opacity-100'

      const styles = apply([chipStateClasses, 'hover:z-50', 'absolute'])

      return html`
        <div class=${`chip -ml-[40px] ${this.toggleChipPosition}`}>
          <stencila-ui-node-chip
            class=${styles}
            style=${nodeDisplay === 'block' ? 'top: 0px;' : ''}
            type=${node}
            node-display=${nodeDisplay}
            ?card-open=${this.toggle}
            .clickEvent=${this.toggleChip.bind(this)}
          >
          </stencila-ui-node-chip>
        </div>
      `
    }

    // protected override update(
    //   // eslint-disable-next-line @typescript-eslint/no-explicit-any
    //   changedProperties: PropertyValueMap<any> | Map<PropertyKey, unknown>
    // ): void {
    //   super.update(changedProperties)

    //   if (changedProperties.has('docViewContext')) {
    //     if (this.docViewContext.nodeChipState === 'hidden' && this.toggle) {
    //       // collapse open container if `nodeChipState` changes to 'hidden'
    //       this.toggleChip()
    //     }
    //   }
    // }
  }

  return ToggleMixin as unknown as Constructor<ChipToggleInterface> & T
}
