import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../../twind'

/**
 * UI Execution Icon
 *
 * Icon displayed as part of row-level layout
 */
@customElement('stencila-ui-execution-icon')
@withTwind()
export class UIExecutionIcon extends LitElement {
  /**
   * The name of the icon to use. We're assuming we're only using the "stencila"
   * shoelace icon library.
   */
  @property({ attribute: 'icon-name' })
  iconName: string = ''

  /**
   * The colour to fill the icon with. This is a valid tailwind colour.
   */
  @property({ attribute: 'fill-colour' })
  fillColour: string = 'black'

  override render() {
    return html`<sl-icon
      name=${this.iconName}
      library="stencila"
      class="text-lg fill-${this.fillColour}"
    ></sl-icon>`
  }
}
