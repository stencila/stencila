import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../twind'
import { IconName } from '../icons/icon'

/**
 * renders a shoelace icon inside a button element.
 * for simple buttons with a single clickable event.
 * for more functionality use `<stencila-ui-simple-icon-button>`
 */
@customElement('stencila-ui-simple-icon-button')
@withTwind()
export class IconButton extends LitElement {
  /**
   * icon name
   */
  @property({ type: String })
  name: IconName

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
        <stencila-ui-icon class=${this.customClasses} name=${this.name}>
        </stencila-ui-icon>
      </button>
    `
  }
}
