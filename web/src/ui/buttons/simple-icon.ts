import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../twind'

import '@shoelace-style/shoelace/dist/components/icon/icon'

/**
 * renders a shoelace icon inside a button element.
 * for simple buttons with a single clickable event.
 * for more fucntionality use `<stencila-ui-simple-icon-button>`
 */
@customElement('stencila-ui-simple-icon-button')
@withTwind()
export class IconButton extends LitElement {
  /**
   * icon name
   */
  @property({ type: String })
  name: string

  /**
   * icon library for shoelace to use
   */
  @property({ type: String })
  library: string = 'default'

  /**
   * custom utility classes to be applied to the icon
   */
  @property({ type: String, attribute: 'custom-classes' })
  customClasses: string

  @property({ type: Boolean })
  disabled: boolean = false

  @property()
  clickEvent: (e: Event) => void | undefined

  override render() {
    return html`
      <button
        class="flex items-center cursor-pointer hover:text-gray-900"
        @click=${this.clickEvent}
        ?disabled=${this.disabled}
      >
        <sl-icon
          class=${this.customClasses}
          name=${this.name}
          library=${this.library}
        >
        </sl-icon>
      </button>
    `
  }
}
