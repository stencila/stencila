import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import '@shoelace-style/shoelace/dist/components/icon/icon'

import StencilaElement from '../utils/element'

// prettier-ignore
const icons: Record<string, string> = {
  'arrow-repeat': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/arrow-repeat.svg'),
  'book': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/book.svg'),
  'bug': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/bug.svg'),
  'braces-asterisk': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/braces-asterisk.svg'),
  'check-circle': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/check-circle.svg'),
  'circle': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/circle.svg'),
  'clock': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/clock.svg'),
  'code-square': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/code-square.svg'),
  'code': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/code.svg'),
  'dash-circle': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/dash-circle.svg'),
  'dash': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/dash.svg'),
  'exclamation-octagon': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/exclamation-octagon.svg'),
  'exclamation-triangle': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/exclamation-triangle.svg'),
  'eye-slash': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/eye-slash.svg'),
  'eye': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/eye.svg'),
  'eyeglasses': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/eyeglasses.svg'),
  'file': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/file.svg'),
  'hourglass': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/hourglass.svg'),
  'house': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/house.svg'),
  'info-circle': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/info-circle.svg'),
  'lightning-fill': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/lightning-fill.svg'),
  'list': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/list.svg'),
  'map': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/map.svg'),
  'palette': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/palette.svg'),
  'pen': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/pen.svg'),
  'pencil': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/pencil.svg'),
  'play': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/play.svg'),
  'search': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/search.svg'),
  'sliders': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/sliders.svg'),
  'stars': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/stars.svg'),
  'terminal-fill': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/terminal-fill.svg'),
  'three-dots-vertical': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/three-dots-vertical.svg'),
  'wifi-off': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/wifi-off.svg'),
  'wifi': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/wifi.svg'),
  'wrench-adjustable': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/wrench-adjustable.svg'),
  'x-lg': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/x-lg.svg'),
}

export type IconName = keyof typeof icons

/**
 * Get the SVG source for an icon as a DataURI
 */
export function getIconSrc(name: IconName): string {
  const svg = encodeURIComponent(icons[name] ?? icons.circle)
  return `data:image/svg+xml,${svg}`
}

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
    return html`<sl-icon
      src=${getIconSrc(this.name)}
      label=${this.label ? this.label : ''}
    ></sl-icon>`
  }
}
