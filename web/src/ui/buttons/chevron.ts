import '@shoelace-style/shoelace/dist/components/icon/icon'

import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../twind'

type ChevronPosition = 'left' | 'down' | 'right'

/**
 * A chevron style button used for collapsing and expanding elements,
 *
 * Will roatate 90 degrees when toggled based
 */
@customElement('stencila-chevron-button')
@withTwind()
export class Chevron extends LitElement {
  @property({ type: Boolean })
  disabled: boolean = false

  @property()
  clickEvent: (e: Event) => void | undefined

  @property({ type: String, attribute: 'default-pos' })
  direction: Exclude<ChevronPosition, 'down'> = 'left'

  @property()
  position: ChevronPosition

  @property({ type: String, attribute: 'custom-class' })
  customClass: string

  @property({ type: String })
  colour: string = 'black'

  private changePosition = () => {
    this.position === this.direction
      ? (this.position = 'down')
      : (this.position = this.direction)
  }

  override render() {
    if (!this.position) {
      this.position = this.direction
    }

    const rotation = this.direction === 'left' ? '-rotate-90' : 'rotate-90'

    const styles = apply([
      this.position === 'down' ? rotation : '',
      'transition-transform duration-100',
    ])

    const icon = `chevron-${this.direction}`

    return html`
      <button
        class="${this.customClass} cursor-pointer leading-[0px]"
        @click=${(e: Event) => {
          this.changePosition()
          this.clickEvent(e)
        }}
      >
        <sl-icon
          class="text-${this.colour} ${styles}"
          name=${icon}
          library="default"
        ></sl-icon>
      </button>
    `
  }
}
