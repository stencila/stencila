import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import '@shoelace-style/shoelace/dist/components/icon/icon'

import StencilaElement from './element'

const icons: Record<string, string> = {
  'arrow-repeat': require('@shoelace-style/shoelace/dist/assets/icons/arrow-repeat.svg'),
  'check-circle': require('@shoelace-style/shoelace/dist/assets/icons/check-circle.svg'),
  'dash-circle': require('@shoelace-style/shoelace/dist/assets/icons/dash-circle.svg'),
  'eye-slash': require('@shoelace-style/shoelace/dist/assets/icons/eye-slash.svg'),
  'lightning-fill': require('@shoelace-style/shoelace/dist/assets/icons/lightning-fill.svg'),
  clock: require('@shoelace-style/shoelace/dist/assets/icons/clock.svg'),
  code: require('@shoelace-style/shoelace/dist/assets/icons/code.svg'),
  dash: require('@shoelace-style/shoelace/dist/assets/icons/dash.svg'),
  eye: require('@shoelace-style/shoelace/dist/assets/icons/eye.svg'),
  hourglass: require('@shoelace-style/shoelace/dist/assets/icons/hourglass.svg'),
}

/**
 * An icon
 *
 * This is a thin wrapper around the Shoelace `<sl-icon>` which, instead of downloading
 * icon SVG files on the fly, bundles them. This avoids having to serve the icons SVGs
 * independently which in turn simplifies embedding components in Stencila binaries.
 */
@customElement('stencila-icon')
export default class StencilaIcon extends StencilaElement {
  /**
   * The name of the icon to render
   */
  @property()
  name: keyof typeof icons

  /**
   * An alternate description to use for accessibility.
   * If omitted, the icon will be ignored by assistive devices.
   */
  @property()
  label?: string

  render() {
    const svg = icons[this.name] ?? icons.circle
    return html`<sl-icon
      src="${svg}"
      ${this.label ? `label=${this.label}` : ''}
    ></sl-icon>`
  }
}
