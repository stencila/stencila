import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../../twind'

const TOOLTIP_OFFSET_Y: number = 10

@customElement('authorship-tooltip')
@withTwind()
export class AuthorshipTooltip extends LitElement {
  @property({ type: Boolean })
  open: boolean = false

  @property()
  xPos: number
  @property()
  yPos: number

  @property()
  content: string

  override render() {
    const styles = apply([
      `fixed top-[${this.yPos - TOOLTIP_OFFSET_Y}px] left-[${this.xPos}px] z-50`,
      this.open ? 'opacity-100' : 'opacity-0',
      'w-32',
      'p-2',
      'font-sans text-white text-sm',
      'rounded drop-shadow',
      'bg-black',
      'transition-all delay-200 duration-300',
      'transform -translate-y-full -translate-x-1/2',
      'pointer-events-none',
      'after:content-[""]',
      'after:absolute after:-bottom-1 after:left-1/2',
      'after:w-2 after:h-2',
      'after:bg-black',
      'after:transform after:-translate-x-1/2 after:rotate-45',
    ])

    return html`<div class=${styles}>${this.content}</div>`
  }
}
