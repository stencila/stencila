import { apply } from '@twind/core'
import { html, LitElement } from 'lit'
import { state } from 'lit/decorators'

import { nodeUi } from '../icons-and-colours'

export declare class ChipToggleInterface {
  protected renderChip: (icons: [string, string], colours: NodeColours) => void
  protected toggle: boolean
  protected toggleChipPosition: string
  protected toggleChip: () => void
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type Constructor<T> = new (...args: any[]) => T
type NodeColours = Pick<ReturnType<typeof nodeUi>, 'borderColour' | 'colour'>

/**
 * A Mixin that provides a "chip" to allow for a card to have its visibility
 * toggled on and off.
 */
export const ToggleChipMixin = <T extends Constructor<LitElement>>(
  superClass: T
) => {
  class ToggleMixin extends superClass {
    @state()
    protected toggle: boolean = false

    /**
     * Used to allow clients to override css classes (tailwind) to change the
     * positioning of the chip.
     */
    protected toggleChipPosition: string = ''

    protected toggleChip() {
      this.toggle = !this.toggle
    }

    protected renderChip(icons: [string, string], colours: NodeColours) {
      const { colour, borderColour } = colours
      const [library, icon] = icons

      const styles = apply([
        this.toggle && 'pointer-events-none',
        !this.toggle && 'group-hover:opacity-100',
        'h-8',
        'flex items-center',
        'opacity-0',
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
        <div class=${`chip -ml-[40px] pr-[6px] ${this.toggleChipPosition}`}>
          <div class=${`${styles}`} @click=${this.toggleChip}>
            <sl-icon
              library=${library}
              name=${icon}
              class="text-base"
            ></sl-icon>
          </div>
        </div>
      `
    }
  }

  return ToggleMixin as unknown as Constructor<ChipToggleInterface> & T
}
