import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import '@shoelace-style/shoelace/dist/components/icon-button/icon-button'

import StencilaElement from './element'
import { getIconSrc, IconName } from './icon'

/**
 * An icon button
 *
 * This is a thin wrapper around the Shoelace `<sl-icon-button>` to reuse the
 * icon set defined for `StencilaIcon`.
 */
@customElement('stencila-icon-button')
export default class StencilaIconButton extends StencilaElement {
  /**
   * The name of the icon to render
   */
  @property()
  name: IconName

  /**
   * An alternate description to use for accessibility.
   * If omitted, the icon will be ignored by assistive devices.
   */
  @property()
  label?: string

  render() {
    return html`<sl-icon-button
      src="${getIconSrc(this.name)}"
      ${this.label ? `label=${this.label}` : ''}
    ></sl-icon-button>`
  }
}
