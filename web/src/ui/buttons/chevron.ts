import '@shoelace-style/shoelace/dist/components/icon/icon'

import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../twind'

@customElement('stencila-chevron-button')
@withTwind()
export class Chevron extends LitElement {
  @property({ type: Boolean })
  disabled: boolean = false

  @property()
  clickEvent: (e: Event) => void | undefined

  @property({ type: String })
  position: 'left' | 'down' = 'left'

  @property({ type: String, attribute: 'custom-classes' })
  customClasses: string

  private changePosition = () => {
    this.position === 'left'
      ? (this.position = 'down')
      : (this.position = 'left')
  }

  override render() {
    const styles = apply([
      this.position === 'down' ? '-rotate-90' : '',
      'transition-transform duration-100',
    ])
    return html`
      <button
        class="${styles} ${this.customClasses} cursor-pointer"
        @click=${(e: Event) => {
          this.changePosition()
          this.clickEvent(e)
        }}
      >
        <sl-icon
          class="text-base"
          name="chevron-left"
          library="default"
        ></sl-icon>
      </button>
    `
  }
}
