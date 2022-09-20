import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import '@shoelace-style/shoelace/dist/components/icon/icon'

import StencilaElement from './element'

// prettier-ignore
const icons: Record<string, string> = {
  'arrow-repeat': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/arrow-repeat.svg'),
  'check-circle': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/check-circle.svg'),
  'circle': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/circle.svg'),
  'clock': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/clock.svg'),
  'code': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/code.svg'),
  'dash-circle': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/dash-circle.svg'),
  'dash': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/dash.svg'),
  'eye-slash': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/eye-slash.svg'),
  'eye': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/eye.svg'),
  'hourglass': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/hourglass.svg'),
  'lightning-fill': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/lightning-fill.svg'),
}

export type IconName = keyof typeof icons

/**
 * An icon
 *
 * This is a thin wrapper around the Shoelace `<sl-icon>` which, instead of downloading
 * icon SVG files on the fly, bundles them as SVG strings. This avoids having to serve the icons SVGs
 * independently which in turn simplifies embedding components in Stencila binaries and
 * reduces the number distributed files and requests.
 */
@customElement('stencila-icon')
export default class StencilaIcon extends StencilaElement {
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
    const svg = encodeURIComponent(icons[this.name] ?? icons.circle)
    return html`<sl-icon
      src="data:image/svg+xml,${svg}"
      ${this.label ? `label=${this.label}` : ''}
    ></sl-icon>`
  }
}
