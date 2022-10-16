import { html } from 'lit'
import { customElement, property } from 'lit/decorators'
import { twSheet } from '../utils/css'

import StencilaElement from '../utils/element'
import { getIconSrc, IconName } from './icon'

const { tw, sheet } = twSheet()

/**
 * An icon button
 *
 * Similar to a Shoelace `<sl-icon-button>` but with different styling.
 */
@customElement('stencila-icon-button')
export default class StencilaIconButton extends StencilaElement {
  static styles = sheet.target

  /**
   * The name of the icon to render
   */
  @property()
  name: IconName

  /**
   * The color of the button
   */
  @property()
  color: string = 'gray'

  /**
   * Additional Tailwind utility classes to add to the button
   */
  @property()
  adjust: string = ''

  /**
   * An alternate description to use for accessibility.
   * If omitted, the icon will be ignored by assistive devices.
   */
  @property()
  label?: string

  render() {
    return html`<span
      class=${tw`flex items-center p-1 rounded-full outline-none cursor-pointer bg-${this.color}-200(hover:& focus:&) focus:ring(1 ${this.color}-300) text-${this.color}-800 ${this.adjust}`}
      tabindex="0"
      role="button"
    >
      <stencila-icon name=${this.name} label=${this.label}> </stencila-icon>
    </span>`
  }
}
