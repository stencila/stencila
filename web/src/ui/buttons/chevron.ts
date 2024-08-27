import '@shoelace-style/shoelace/dist/components/icon/icon'

import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../twind'

type ChevronPosition = 'left' | 'down' | 'right' | 'up'

/**
 * A chevron style button used for collapsing and expanding elements,
 */
@customElement('stencila-ui-chevron-button')
@withTwind()
export class ChevronButton extends LitElement {
  @property({ type: Boolean })
  disabled: boolean = false

  @property()
  clickEvent?: (e: Event) => void

  @property({ type: String, attribute: 'default-pos' })
  direction: ChevronPosition = 'down'

  @property()
  position: ChevronPosition

  @property({ type: String, attribute: 'custom-class' })
  customClass: string

  @property({ type: String })
  colour: string = 'black'

  @property({ type: Boolean })
  disableEvents?: boolean = false

  private changePosition = () => {
    this.position = this.direction
      ? (this.position = 'down')
      : (this.position = this.direction)
  }

  override render() {
    if (!this.position) {
      this.position = this.direction
    }

    let rotation = ''

    switch (this.direction) {
      case 'left':
        rotation = 'rotate-90'
        break
      case 'up':
        rotation = 'rotate-180'
        break
      case 'down':
        rotation = 'rotate-0'
        break
      default:
        rotation = '-rotate-90'
        break
    }

    const styles = apply([rotation, 'transition-transform duration-100'])

    return html`
      <button
        class="${this.customClass} cursor-pointer leading-[0px]"
        @click=${(e: Event) => {
          if (this.disableEvents) {
            return
          }

          this.changePosition()
          if (this.clickEvent) {
            this.clickEvent(e)
          }
        }}
      >
        <stencila-ui-icon
          class="${styles}"
          name="chevronDown"
        ></stencila-ui-icon>
      </button>
    `
  }
}
