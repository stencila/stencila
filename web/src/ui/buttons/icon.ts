import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../twind'
import { IconName } from '../icons/icon'

/**
 * Renders a Shoelace icon inside a button element.
 * For simple buttons with a single clickable event.
 */
@customElement('stencila-ui-icon-button')
@withTwind()
export class UIIconButton extends LitElement {
  /**
   * The name of the icon
   */
  @property({ type: String })
  name: IconName

  /**
   * Whether the button is disabled
   */
  @property({ type: Boolean })
  disabled: boolean = false

  override render() {
    const style = apply([
      'flex items-center hover:text-gray-900',
      this.disabled ? 'cursor-not-allowed' : 'cursor-pointer',
    ])

    return html`
      <button class=${style} ?disabled=${this.disabled}>
        <stencila-ui-icon name=${this.name}> </stencila-ui-icon>
      </button>
    `
  }
}
