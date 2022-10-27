import { css, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import '@shoelace-style/shoelace/dist/components/icon/icon'

import StencilaElement from '../utils/element'

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
   * Styles to ensure that the the size of the <stencila-icon> is the same as the
   * <sl-icon> that it wraps
   */
  static styles = css`
    :host {
      display: inline-block;
      width: 1em;
      height: 1em;
      line-height: 1;
      contain: strict;
      box-sizing: content-box !important;
    }
  `
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
      label=${this.label}
    ></sl-icon>`
  }
}

export type IconName = keyof typeof icons

/**
 * Get the SVG source for an icon as a DataURI
 */
export function getIconSrc(name: IconName): string {
  let svg = icons[name]
  if (svg === undefined && name.endsWith('-color')) {
    svg = icons[name.replace('-color', '')]
  }
  if (svg === undefined) {
    console.warn(`No icon with name "${name}"`)
  }
  return `data:image/svg+xml,${encodeURIComponent(svg ?? icons.circle)}`
}

const calculator = require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/calculator.svg')
const questionSquare = require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/question-square.svg')
const globe = require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/globe.svg')

// prettier-ignore
const icons: Record<string, string> = {
  // Bootstrap icons https://icons.getbootstrap.com/ (that come with Shoelace)
  'arrow-bar-right': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/arrow-bar-right.svg'),
  'arrow-clockwise': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/arrow-clockwise.svg'),
  'arrow-down': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/arrow-down.svg'),
  'arrow-repeat': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/arrow-repeat.svg'),
  'arrow-return-right': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/arrow-return-right.svg'),
  'arrow-right': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/arrow-right.svg'),
  'arrow-up': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/arrow-up.svg'),
  'book': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/book.svg'),
  'box-arrow-in-right': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/box-arrow-in-right.svg'),
  'braces-asterisk': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/braces-asterisk.svg'),
  'broadcast-pin': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/broadcast-pin.svg'),
  'brush': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/brush.svg'),
  'bug': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/bug.svg'),
  'check-circle': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/check-circle.svg'),
  'chevron-down': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/chevron-down.svg'),
  'chevron-right': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/chevron-right.svg'),
  'chevron-left': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/chevron-left.svg'),
  'circle': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/circle.svg'),
  'clipboard': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/clipboard.svg'),
  'clipboard-check': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/clipboard-check.svg'),
  'clock': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/clock.svg'),
  'code-square': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/code-square.svg'),
  'code': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/code.svg'),
  'dash-circle': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/dash-circle.svg'),
  'dash': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/dash.svg'),
  'download': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/download.svg'),
  'exclamation-circle': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/exclamation-circle.svg'),
  'exclamation-octagon': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/exclamation-octagon.svg'),
  'exclamation-triangle': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/exclamation-triangle.svg'),
  'eye-slash': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/eye-slash.svg'),
  'eye': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/eye.svg'),
  'eyeglasses': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/eyeglasses.svg'),
  'file-plus': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/file-plus.svg'),
  'file': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/file.svg'),
  'filter-square': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/filter-square.svg'),
  'filter': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/filter.svg'),
  'gear': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/gear.svg'),
  'globe': globe,
  'hourglass': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/hourglass.svg'),
  'house': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/house.svg'),
  'info-circle': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/info-circle.svg'),
  'keyboard': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/keyboard.svg'),
  'lightning': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/lightning.svg'),
  'lightning-fill': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/lightning-fill.svg'),
  'list': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/list.svg'),
  'list-nested': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/list-nested.svg'),
  'map': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/map.svg'),
  'magic': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/magic.svg'),
  'markdown': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/markdown.svg'),
  'palette': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/palette.svg'),
  'pen': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/pen.svg'),
  'pencil': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/pencil.svg'),
  'play': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/play.svg'),
  'plus-lg': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/plus-lg.svg'),
  'question-circle': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/question-circle.svg'),
  'question-square': questionSquare,
  'repeat': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/repeat.svg'),
  'search': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/search.svg'),
  'slash-circle': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/slash-circle.svg'),
  'sliders': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/sliders.svg'),
  'stars': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/stars.svg'),
  'stopwatch': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/stopwatch.svg'),
  'terminal-fill': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/terminal-fill.svg'),
  'trash2': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/trash2.svg'),
  'three-dots-vertical': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/three-dots-vertical.svg'),
  'wifi-off': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/wifi-off.svg'),
  'wifi': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/wifi.svg'),
  'wrench-adjustable': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/wrench-adjustable.svg'),
  'x-circle': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/x-circle.svg'),
  'x-lg': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/x-lg.svg'),

  // File Icons https://github.com/file-icons/icons
  'asciimath': '<svg xmlns="http://www.w3.org/2000/svg" width="0.86em" height="1em" preserveAspectRatio="xMidYMid meet" viewBox="0 0 439 512"><path fill="currentColor" d="M438.857 128v347.429c-.385 10.285-4 18.857-10.857 25.714S412.571 511.615 402.286 512H36.57c-10.285-.385-18.857-4-25.714-10.857S.373 485.714 0 475.429V36.57c.373-10.285 4-18.857 10.857-25.714S26.286.385 36.571 0h274.286l128 128zm-36.571 18.286L292.57 36.57h-256v438.86h365.715V146.286zm-96.782 199.04H186.187l-24.766 29.761h11.904c11.746 0 19.14 2.266 22.187 6.79c3.044 4.524 2.045 33.005-4.916 37.53c-6.952 4.523-10.362 4.49-22.108 4.49h-77.67c-11.746 0-19.143-2.258-22.19-6.782c-3.04-4.524-3.957-32.646 1.714-36.276c7.22-4.623 18.404-6.065 30.333-5.752l159.17-190.948h-23.778c-11.746 0-19.142-2.266-22.186-6.79c-3.044-4.524-2.992-10.432.163-17.734c3.154-7.298 8.21-13.214 15.166-17.738c6.953-4.524 16.306-6.79 28.052-6.79l102.145.238v239.762c11.428 0 15.192 1.274 17.748 3.81c5.04 5.238 3.66 38.037.963 40.542c-4.741 4.402-10.343 4.458-17.455 4.458h-81.969c-11.746 0-19.142-2.258-22.19-6.782c-3.04-4.524-4.211-32.285 2.745-36.81c6.953-4.523 20.664-5.218 32.41-5.218h13.845v-29.762zm0-48.806v-96.667l-78.706 96.667h78.706z"/></svg>',

  // Fluent Icons
  'date': '<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" preserveAspectRatio="xMidYMid meet" viewBox="0 0 2048 2048"><path fill="currentColor" d="M2048 128v1792H0V128h384V0h128v128h1024V0h128v128h384zM128 256v256h1792V256h-256v128h-128V256H512v128H384V256H128zm1792 1536V640H128v1152h1792zm-512-896v640h-128v-486q-27 14-62 26t-66 12V960q12 0 31-6t39-15t36-21t22-21v-1h128zm-384 192q0 39-11 70t-31 58t-44 51t-51 46t-51 46t-47 49h235v128H640v-36q0-19-1-38t4-38t10-36q11-27 33-53t50-53t55-51t51-49t39-47t15-47q0-27-19-45t-45-19q-23 0-40 14t-23 37l-125-26q6-33 23-61t44-48t57-32t64-12q40 0 75 15t61 41t41 61t15 75z"/></svg>',
  'date-time': '<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" preserveAspectRatio="xMidYMid meet" viewBox="0 0 2048 2048"><path fill="currentColor" d="M1792 993q60 41 107 93t81 114t50 131t18 141q0 119-45 224t-124 183t-183 123t-224 46q-91 0-176-27t-156-78t-126-122t-85-157H128V128h256V0h128v128h896V0h128v128h256v865zM256 256v256h1408V256h-128v128h-128V256H512v128H384V256H256zm643 1280q-3-31-3-64q0-119 45-224t124-183t183-123t224-46q100 0 192 33V640H256v896h643zm573 384q93 0 174-35t142-96t96-142t36-175q0-93-35-174t-96-142t-142-96t-175-36q-93 0-174 35t-142 96t-96 142t-36 175q0 93 35 174t96 142t142 96t175 36zm64-512h192v128h-320v-384h128v256z"/></svg>',

  // Carbon icons https://github.com/carbon-design-system/carbon/tree/main/packages/icons
  'boolean': '<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" preserveAspectRatio="xMidYMid meet" viewBox="0 0 32 32"><path fill="currentColor" d="M23 23a7 7 0 1 1 7-7a7.008 7.008 0 0 1-7 7Zm0-12a5 5 0 1 0 5 5a5.005 5.005 0 0 0-5-5Z"/><circle cx="9" cy="16" r="7" fill="currentColor"/></svg>',
  'integer': '<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" preserveAspectRatio="xMidYMid meet" viewBox="0 0 32 32"><path fill="currentColor" d="M26 12h-4v2h4v2h-3v2h3v2h-4v2h4a2.003 2.003 0 0 0 2-2v-6a2.002 2.002 0 0 0-2-2zm-7 10h-6v-4a2.002 2.002 0 0 1 2-2h2v-2h-4v-2h4a2.002 2.002 0 0 1 2 2v2a2.002 2.002 0 0 1-2 2h-2v2h4zM8 20v-8H6v1H4v2h2v5H4v2h6v-2H8z"/></svg>',
  'number': '<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" preserveAspectRatio="xMidYMid meet" viewBox="0 0 32 32"><path fill="currentColor" d="M21 15h2v2h-2z"/><path fill="currentColor" d="M24 23h-4a2.002 2.002 0 0 1-2-2V11a2.002 2.002 0 0 1 2-2h4a2.002 2.002 0 0 1 2 2v10a2.003 2.003 0 0 1-2 2zm-4-12v10h4V11zm-9 4h2v2h-2z"/><path fill="currentColor" d="M14 23h-4a2.002 2.002 0 0 1-2-2V11a2.002 2.002 0 0 1 2-2h4a2.002 2.002 0 0 1 2 2v10a2.003 2.003 0 0 1-2 2zm-4-12v10h4V11zM4 21h2v2H4z"/></svg>',
  'string': '<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" preserveAspectRatio="xMidYMid meet" viewBox="0 0 32 32"><path fill="currentColor" d="M29 22h-5a2.003 2.003 0 0 1-2-2v-6a2.002 2.002 0 0 1 2-2h5v2h-5v6h5zM18 12h-4V8h-2v14h6a2.003 2.003 0 0 0 2-2v-6a2.002 2.002 0 0 0-2-2zm-4 8v-6h4v6zm-6-8H3v2h5v2H4a2 2 0 0 0-2 2v2a2 2 0 0 0 2 2h6v-8a2.002 2.002 0 0 0-2-2zm0 8H4v-2h4z"/></svg>',

  // Codicons https://github.com/microsoft/vscode-codicons
  'call-outgoing': '<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" preserveAspectRatio="xMidYMid meet" viewBox="0 0 16 16"><path fill="currentColor" d="M8.648 6.648L13.29 2H10V1h5v5h-1V2.71L9.352 7.353l-.704-.704zm3.305 2.563a1.567 1.567 0 0 1 1.102.453c.11.11.232.224.367.344l.43.383c.15.135.291.276.421.421c.13.146.253.295.368.446c.114.15.2.312.257.484c.058.172.092.344.102.516a1.568 1.568 0 0 1-.453 1.101c-.266.266-.505.5-.719.704a4.006 4.006 0 0 1-.664.515c-.23.14-.487.245-.773.313a4.696 4.696 0 0 1-1.008.109a6.157 6.157 0 0 1-1.75-.266A9.819 9.819 0 0 1 7.843 14a12.445 12.445 0 0 1-1.741-1.117a15.329 15.329 0 0 1-1.61-1.414c-.505-.51-.974-1.05-1.406-1.617a11.64 11.64 0 0 1-1.102-1.735a10.38 10.38 0 0 1-.718-1.773A6.005 6.005 0 0 1 1 4.625c0-.396.034-.734.102-1.016a2.63 2.63 0 0 1 .312-.765c.14-.23.313-.45.516-.664c.203-.214.44-.456.71-.727A1.567 1.567 0 0 1 3.743 1c.26 0 .51.07.75.21c.24.142.472.313.696.517c.223.203.43.416.617.64c.187.224.364.417.53.578a1.567 1.567 0 0 1 .453 1.102a1.4 1.4 0 0 1-.1.547a1.824 1.824 0 0 1-.25.43a2.983 2.983 0 0 1-.329.351c-.12.11-.229.214-.328.313a3.128 3.128 0 0 0-.258.289a.46.46 0 0 0-.101.312c0 .063.047.162.14.297a5.3 5.3 0 0 0 .391.484a24.386 24.386 0 0 0 1.211 1.266c.234.23.461.45.68.664c.218.214.43.417.633.61c.203.192.375.356.515.492c.14.135.25.24.328.312a.534.534 0 0 0 .696.063c.093-.068.19-.152.289-.25c.099-.1.2-.209.304-.329c.104-.12.224-.229.36-.328c.135-.099.278-.185.43-.258a1.21 1.21 0 0 1 .554-.101zM11.383 14c.318 0 .583-.029.797-.086a1.93 1.93 0 0 0 .586-.266c.177-.12.343-.26.5-.421c.156-.162.346-.352.57-.57c.11-.11.164-.24.164-.391c0-.068-.042-.164-.125-.29a6.122 6.122 0 0 0-.313-.421a5.01 5.01 0 0 0-.43-.47c-.16-.155-.317-.299-.468-.429a4.322 4.322 0 0 0-.414-.32c-.125-.083-.224-.125-.297-.125a.545.545 0 0 0-.312.101a1.801 1.801 0 0 0-.29.25c-.093.1-.195.209-.304.329c-.11.12-.23.229-.36.328c-.13.099-.273.185-.43.258a1.208 1.208 0 0 1-.546.101a1.527 1.527 0 0 1-1.102-.46L4.883 7.39a1.537 1.537 0 0 1-.336-.5a1.655 1.655 0 0 1-.125-.602c0-.203.034-.383.101-.539c.068-.156.151-.302.25-.438c.1-.135.209-.252.329-.351c.12-.099.229-.203.328-.313c.099-.109.185-.205.258-.289a.429.429 0 0 0 .101-.312c0-.068-.042-.164-.125-.29a5.085 5.085 0 0 0-.312-.413a6.791 6.791 0 0 0-.43-.469a6.787 6.787 0 0 0-.469-.43a5.674 5.674 0 0 0-.422-.32c-.13-.089-.226-.13-.289-.125a.542.542 0 0 0-.398.164a65.24 65.24 0 0 1-.57.563a3.073 3.073 0 0 0-.422.5a1.9 1.9 0 0 0-.258.586A3.377 3.377 0 0 0 2 4.601c0 .5.08 1.015.242 1.546a9.12 9.12 0 0 0 .672 1.61c.287.541.63 1.068 1.031 1.578c.401.51.831.997 1.29 1.46a13.205 13.205 0 0 0 3.046 2.298a8.37 8.37 0 0 0 1.586.664a5.526 5.526 0 0 0 1.516.242z"/></svg>',

  // Material design icons https://github.com/Templarian/MaterialDesign
  'mathml': '<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" preserveAspectRatio="xMidYMid meet" viewBox="0 0 24 24"><path fill="currentColor" d="m12.89 3l1.96.4L11.11 21l-1.96-.4L12.89 3m6.7 9L16 8.41V5.58L22.42 12L16 18.41v-2.83L19.59 12M1.58 12L8 5.58v2.83L4.41 12L8 15.58v2.83L1.58 12Z"/></svg>',
  'code-greater-than': '<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" preserveAspectRatio="xMidYMid meet" viewBox="0 0 24 24"><path fill="currentColor" d="M10.41 7.41L15 12l-4.59 4.6L9 15.18L12.18 12L9 8.82M5 3a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2V5a2 2 0 0 0-2-2H5Z"/></svg>',

  // Phosphor icons https://github.com/phosphor-icons/phosphor-icons
  'brackets': '<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" preserveAspectRatio="xMidYMid meet" viewBox="0 0 256 256"><path fill="currentColor" d="M82.3 222.2A12 12 0 0 1 72 228a11.9 11.9 0 0 1-6.2-1.7C64 225.2 20 198 20 128s44-97.2 45.8-98.3a12 12 0 1 1 12.4 20.5C76.7 51.2 44 72.3 44 128s32.9 76.9 34.3 77.8a12 12 0 0 1 4 16.4ZM190.2 29.7a12 12 0 0 0-12.5 20.5c1.4.9 34.3 22 34.3 77.8s-32.9 76.9-34.2 77.7A12 12 0 0 0 184 228a12.9 12.9 0 0 0 6.2-1.7C192 225.2 236 198 236 128s-44-97.2-45.8-98.3Z"/></svg>',
  'stamp-light': '<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" preserveAspectRatio="xMidYMid meet" viewBox="0 0 256 256"><path fill="currentColor" d="M222 224a6 6 0 0 1-6 6H40a6 6 0 0 1 0-12h176a6 6 0 0 1 6 6Zm0-80v40a14 14 0 0 1-14 14H48a14 14 0 0 1-14-14v-40a14 14 0 0 1 14-14h56.6L88.4 54.3a29.4 29.4 0 0 1 6-25.2A30 30 0 0 1 117.7 18h20.6a30 30 0 0 1 23.3 11.1a29.4 29.4 0 0 1 6 25.2L151.4 130H208a14 14 0 0 1 14 14Zm-105.1-14h22.2l16.8-78.2A18 18 0 0 0 138.3 30h-20.6a18 18 0 0 0-17.6 21.8Zm93.1 14a2 2 0 0 0-2-2H48a2 2 0 0 0-2 2v40a2 2 0 0 0 2 2h160a2 2 0 0 0 2-2Z"/></svg>',

  // Tabler icons https://github.com/tabler/tabler-icons
  'math': '<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" preserveAspectRatio="xMidYMid meet" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 5h-7L8 19l-3-6H3m11 0l6 6m-6 0l6-6"/></svg>',
  'tex': '<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" preserveAspectRatio="xMidYMid meet" viewBox="0 0 24 24"><path fill="none" stroke="currentColor" stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 8V7H3v1m3 7V7m15 8l-5-8m0 8l5-8m-7 4h-4v8h4m-4-4h3"/></svg>',

  // VSCode Icons https://github.com/vscode-icons/vscode-icons
  'css-color': '<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" preserveAspectRatio="xMidYMid meet" viewBox="0 0 32 32"><path fill="#1572b6" d="M5.902 27.201L3.656 2h24.688l-2.249 25.197L15.985 30L5.902 27.201z"/><path fill="#33a9dc" d="m16 27.858l8.17-2.265l1.922-21.532H16v23.797z"/><path fill="#fff" d="M16 13.191h4.09l.282-3.165H16V6.935h7.75l-.074.829l-.759 8.518H16v-3.091z"/><path fill="#ebebeb" d="m16.019 21.218l-.014.004l-3.442-.93l-.22-2.465H9.24l.433 4.853l6.331 1.758l.015-.004v-3.216z"/><path fill="#fff" d="m19.827 16.151l-.372 4.139l-3.447.93v3.216l6.336-1.756l.047-.522l.537-6.007h-3.101z"/><path fill="#ebebeb" d="M16.011 6.935v3.091H8.545l-.062-.695l-.141-1.567l-.074-.829h7.743zM16 13.191v3.091h-3.399l-.062-.695l-.14-1.567l-.074-.829H16z"/></svg>',
  'json': '<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" preserveAspectRatio="xMidYMid meet" viewBox="0 0 32 32"><path fill="#f5de19" d="M4.014 14.976a2.51 2.51 0 0 0 1.567-.518a2.377 2.377 0 0 0 .805-1.358a15.261 15.261 0 0 0 .214-2.944q.012-2.085.075-2.747a5.236 5.236 0 0 1 .418-1.686a3.025 3.025 0 0 1 .755-1.018A3.046 3.046 0 0 1 9 4.125A6.762 6.762 0 0 1 10.544 4h.7v1.96h-.387a2.338 2.338 0 0 0-1.723.468a3.4 3.4 0 0 0-.425 2.092a36.054 36.054 0 0 1-.137 4.133a4.734 4.734 0 0 1-.768 2.06A4.567 4.567 0 0 1 6.1 16a3.809 3.809 0 0 1 1.992 1.754a8.861 8.861 0 0 1 .618 3.865q0 2.435.05 2.9a1.755 1.755 0 0 0 .504 1.181a2.639 2.639 0 0 0 1.592.337h.387V28h-.7a5.655 5.655 0 0 1-1.773-.2a2.97 2.97 0 0 1-1.324-.93a3.353 3.353 0 0 1-.681-1.63a24.175 24.175 0 0 1-.165-3.234a16.469 16.469 0 0 0-.214-3.106a2.408 2.408 0 0 0-.805-1.361a2.489 2.489 0 0 0-1.567-.524Zm23.972 2.035a2.489 2.489 0 0 0-1.567.524a2.408 2.408 0 0 0-.805 1.361a16.469 16.469 0 0 0-.212 3.109a24.175 24.175 0 0 1-.169 3.234a3.353 3.353 0 0 1-.681 1.63a2.97 2.97 0 0 1-1.324.93a5.655 5.655 0 0 1-1.773.2h-.7V26.04h.387a2.639 2.639 0 0 0 1.592-.337a1.755 1.755 0 0 0 .506-1.186q.05-.462.05-2.9a8.861 8.861 0 0 1 .618-3.865A3.809 3.809 0 0 1 25.9 16a4.567 4.567 0 0 1-1.7-1.286a4.734 4.734 0 0 1-.768-2.06a36.054 36.054 0 0 1-.137-4.133a3.4 3.4 0 0 0-.425-2.092a2.338 2.338 0 0 0-1.723-.468h-.387V4h.7a6.762 6.762 0 0 1 1.54.125a3.046 3.046 0 0 1 1.149.581a3.025 3.025 0 0 1 .755 1.018a5.236 5.236 0 0 1 .418 1.686q.062.662.075 2.747a15.261 15.261 0 0 0 .212 2.947a2.377 2.377 0 0 0 .805 1.355a2.51 2.51 0 0 0 1.567.518Z"/></svg>',
  'json5': '<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" preserveAspectRatio="xMidYMid meet" viewBox="0 0 32 32"><path d="m12.815 15.167l.68-5.676h6.489v2h-4.4l-.255 2.209a2.4 2.4 0 0 1 .252-.122a2.962 2.962 0 0 1 .374-.13a2.9 2.9 0 0 1 .458-.106a2.834 2.834 0 0 1 .512-.046a3.983 3.983 0 0 1 1.466.252a2.736 2.736 0 0 1 1.076.723a3.167 3.167 0 0 1 .664 1.168a5 5 0 0 1 .228 1.588a4.157 4.157 0 0 1-.236 1.387a3.307 3.307 0 0 1-1.9 2.029a4.3 4.3 0 0 1-1.715.312a4.75 4.75 0 0 1-1.458-.228a4.054 4.054 0 0 1-1.252-.656a3.359 3.359 0 0 1-.878-1.046a2.787 2.787 0 0 1-.32-1.382h2.58a1.458 1.458 0 0 0 .39.97a1.383 1.383 0 0 0 1.558.206a1.089 1.089 0 0 0 .4-.412a1.749 1.749 0 0 0 .206-.618a4.3 4.3 0 0 0 .062-.74a2.709 2.709 0 0 0-.092-.74a1.506 1.506 0 0 0-.282-.558a1.229 1.229 0 0 0-.5-.349a1.78 1.78 0 0 0-.718-.13a2.121 2.121 0 0 0-.5.054a1.805 1.805 0 0 0-.382.138a1.318 1.318 0 0 0-.274.19a1.28 1.28 0 0 0-.19.2l-2.048-.482"/><path fill="#999" d="M5.985 23.343a4.45 4.45 0 0 1-1.311-.58a3.2 3.2 0 0 1-.848-.824a3.012 3.012 0 0 1-.458-1.008a4.879 4.879 0 0 1-.13-1.143v-1.55a2.3 2.3 0 0 0-.076-.618a1.184 1.184 0 0 0-.222-.466a.969.969 0 0 0-.382-.306A1.324 1.324 0 0 0 2 16.744v-1.732a1.074 1.074 0 0 0 .937-.4a1.841 1.841 0 0 0 .3-1.1v-1.55a4.879 4.879 0 0 1 .13-1.143a3.037 3.037 0 0 1 .458-1.008A3.17 3.17 0 0 1 4.671 9a4.482 4.482 0 0 1 1.311-.58l.48 1.344a1.222 1.222 0 0 0-.488.328a1.742 1.742 0 0 0-.306.5a2.524 2.524 0 0 0-.16.634a5.827 5.827 0 0 0-.046.74v1.55a2.844 2.844 0 0 1-.336 1.384a2.37 2.37 0 0 1-1.076.983a2.356 2.356 0 0 1 1.076.992a2.808 2.808 0 0 1 .336 1.374v1.55a5.827 5.827 0 0 0 .046.74a2.586 2.586 0 0 0 .16.634a1.684 1.684 0 0 0 .306.5a1.222 1.222 0 0 0 .488.327c0-.005-.477 1.344-.477 1.344m2.23-3.951a1.289 1.289 0 0 1 .1-.512a1.212 1.212 0 0 1 .29-.4a1.373 1.373 0 0 1 .45-.274a1.637 1.637 0 0 1 .58-.1a1.555 1.555 0 0 1 .572.1a1.269 1.269 0 0 1 .45.274a1.077 1.077 0 0 1 .29.4a1.294 1.294 0 0 1 0 1.024a1.151 1.151 0 0 1-.29.412a1.388 1.388 0 0 1-.45.268a1.613 1.613 0 0 1-.572.1a1.578 1.578 0 0 1-.58-.1a1.409 1.409 0 0 1-.45-.268a1.229 1.229 0 0 1-.39-.924m0-6.088a1.289 1.289 0 0 1 .1-.512a1.212 1.212 0 0 1 .29-.4a1.373 1.373 0 0 1 .45-.274a1.637 1.637 0 0 1 .58-.1a1.555 1.555 0 0 1 .572.1a1.269 1.269 0 0 1 .45.274a1.077 1.077 0 0 1 .29.4a1.294 1.294 0 0 1 0 1.024a1.151 1.151 0 0 1-.29.412a1.388 1.388 0 0 1-.45.268a1.613 1.613 0 0 1-.572.1a1.578 1.578 0 0 1-.58-.1a1.409 1.409 0 0 1-.45-.268a1.229 1.229 0 0 1-.39-.924m16.025 6.988a3.648 3.648 0 0 1-.122.929a4.534 4.534 0 0 1-.336.891a4.706 4.706 0 0 1-.5.807a4.005 4.005 0 0 1-.61.664l-1.3-.61c.081-.173.168-.349.26-.526a4.846 4.846 0 0 0 .268-.558a4.443 4.443 0 0 0 .206-.656a3.406 3.406 0 0 0 .084-.8v-1.778h2.059l-.008 1.636m1.297 1.702a1.251 1.251 0 0 0 .488-.328a1.707 1.707 0 0 0 .306-.5a2.525 2.525 0 0 0 .16-.634a5.826 5.826 0 0 0 .046-.74v-1.55a2.844 2.844 0 0 1 .336-1.382a2.364 2.364 0 0 1 1.084-.983a2.364 2.364 0 0 1-1.084-.983a2.844 2.844 0 0 1-.336-1.382v-1.55a5.827 5.827 0 0 0-.046-.74a2.586 2.586 0 0 0-.16-.634a1.684 1.684 0 0 0-.306-.5a1.222 1.222 0 0 0-.488-.328l.48-1.338A4.45 4.45 0 0 1 27.329 9a3.092 3.092 0 0 1 .848.815a2.892 2.892 0 0 1 .45 1.008a4.606 4.606 0 0 1 .138 1.143v1.55a2.655 2.655 0 0 0 .068.626a1.448 1.448 0 0 0 .222.474a1.037 1.037 0 0 0 .382.3a1.376 1.376 0 0 0 .564.106v1.731a1.077 1.077 0 0 0-.946.412a1.828 1.828 0 0 0-.29 1.084v1.55a4.606 4.606 0 0 1-.138 1.143a2.915 2.915 0 0 1-.45 1.008a3.157 3.157 0 0 1-.848.824a4.482 4.482 0 0 1-1.311.58l-.48-1.352"/></svg>',
  'yaml': '<svg xmlns="http://www.w3.org/2000/svg" width="1em" height="1em" preserveAspectRatio="xMidYMid meet" viewBox="0 0 32 32"><path fill="#fbc02d" d="M2 12.218c.755 0 1.51-.008 2.264 0l.053.038l2.761 2.758c.891-.906 1.8-1.794 2.7-2.7c.053-.052.11-.113.192-.1h1.823a1.4 1.4 0 0 1 .353.019c-.7.67-1.377 1.369-2.069 2.05L5.545 18.8c-.331.324-.648.663-.989.975c-.754.022-1.511.007-2.266.007c1.223-1.209 2.431-2.433 3.658-3.637c-1.321-1.304-2.63-2.62-3.948-3.927Zm10.7 0h1.839v7.566c-.611 0-1.222.012-1.832-.008v-4.994c-1.6 1.607-3.209 3.2-4.811 4.8c-.089.08-.166.217-.305.194c-.824-.006-1.649 0-2.474 0Q8.916 16 12.7 12.218Zm2.258.002c.47-.009.939 0 1.409 0c.836.853 1.69 1.689 2.536 2.532q1.268-1.267 2.539-2.532h1.4q-.008 3.784 0 7.567c-.471 0-.943.006-1.414 0q.008-2.387 0-4.773c-.844.843-1.676 1.7-2.526 2.536c-.856-.835-1.687-1.695-2.532-2.541c0 1.594-.006 3.188.006 4.781c-.472 0-.943.005-1.415 0q-.003-3.79-.003-7.57Zm8.301-.003c.472 0 .944-.007 1.416 0q-.007 3.083 0 6.166h3.782c.063.006.144-.012.191.045c.448.454.907.9 1.353 1.354q-3.371.007-6.741 0q.007-3.782-.001-7.565Z"/></svg>',

  // Language icons, mostly from https://devicon.dev/ but some from project sites and Remix Icon
  'bash': '<svg viewBox="0 0 128 128"><path fill="none" d="M-143.76 4.24h119.53v119.53h-119.53z"></path><path fill="#293138" d="M109.01 28.64L71.28 6.24c-2.25-1.33-4.77-2-7.28-2s-5.03.67-7.28 2.01l-37.74 22.4c-4.5 2.67-7.28 7.61-7.28 12.96v44.8c0 5.35 2.77 10.29 7.28 12.96l37.73 22.4c2.25 1.34 4.76 2 7.28 2 2.51 0 5.03-.67 7.28-2l37.74-22.4c4.5-2.67 7.28-7.62 7.28-12.96V41.6c0-5.34-2.77-10.29-7.28-12.96zM79.79 98.59l.06 3.22c0 .39-.25.83-.55.99l-1.91 1.1c-.3.15-.56-.03-.56-.42l-.03-3.17c-1.63.68-3.29.84-4.34.42-.2-.08-.29-.37-.21-.71l.69-2.91c.06-.23.18-.46.34-.6.06-.06.12-.1.18-.13.11-.06.22-.07.31-.03 1.14.38 2.59.2 3.99-.5 1.78-.9 2.97-2.72 2.95-4.52-.02-1.64-.9-2.31-3.05-2.33-2.74.01-5.3-.53-5.34-4.57-.03-3.32 1.69-6.78 4.43-8.96l-.03-3.25c0-.4.24-.84.55-1l1.85-1.18c.3-.15.56.04.56.43l.03 3.25c1.36-.54 2.54-.69 3.61-.44.23.06.34.38.24.75l-.72 2.88c-.06.22-.18.44-.33.58a.77.77 0 01-.19.14c-.1.05-.19.06-.28.05-.49-.11-1.65-.36-3.48.56-1.92.97-2.59 2.64-2.58 3.88.02 1.48.77 1.93 3.39 1.97 3.49.06 4.99 1.58 5.03 5.09.05 3.44-1.79 7.15-4.61 9.41zm19.78-5.41c0 .3-.04.58-.29.72l-9.54 5.8c-.25.15-.45.02-.45-.28v-2.46c0-.3.18-.46.43-.61l9.4-5.62c.25-.15.45-.02.45.28v2.17zm6.56-55.09l-35.7 22.05c-4.45 2.6-7.73 5.52-7.74 10.89v43.99c0 3.21 1.3 5.29 3.29 5.9-.65.11-1.32.19-1.98.19-2.09 0-4.15-.57-5.96-1.64l-37.73-22.4c-3.69-2.19-5.98-6.28-5.98-10.67V41.6c0-4.39 2.29-8.48 5.98-10.67l37.74-22.4c1.81-1.07 3.87-1.64 5.96-1.64s4.15.57 5.96 1.64l37.74 22.4c3.11 1.85 5.21 5.04 5.8 8.63-1.27-2.67-4.09-3.39-7.38-1.47z"></path></svg>',
  'calc': calculator,
  'javascript-color': '<svg viewBox="0 0 128 128"><path fill="#F0DB4F" d="M1.408 1.408h125.184v125.185H1.408z"></path><path fill="#323330" d="M116.347 96.736c-.917-5.711-4.641-10.508-15.672-14.981-3.832-1.761-8.104-3.022-9.377-5.926-.452-1.69-.512-2.642-.226-3.665.821-3.32 4.784-4.355 7.925-3.403 2.023.678 3.938 2.237 5.093 4.724 5.402-3.498 5.391-3.475 9.163-5.879-1.381-2.141-2.118-3.129-3.022-4.045-3.249-3.629-7.676-5.498-14.756-5.355l-3.688.477c-3.534.893-6.902 2.748-8.877 5.235-5.926 6.724-4.236 18.492 2.975 23.335 7.104 5.332 17.54 6.545 18.873 11.531 1.297 6.104-4.486 8.08-10.234 7.378-4.236-.881-6.592-3.034-9.139-6.949-4.688 2.713-4.688 2.713-9.508 5.485 1.143 2.499 2.344 3.63 4.26 5.795 9.068 9.198 31.76 8.746 35.83-5.176.165-.478 1.261-3.666.38-8.581zM69.462 58.943H57.753l-.048 30.272c0 6.438.333 12.34-.714 14.149-1.713 3.558-6.152 3.117-8.175 2.427-2.059-1.012-3.106-2.451-4.319-4.485-.333-.584-.583-1.036-.667-1.071l-9.52 5.83c1.583 3.249 3.915 6.069 6.902 7.901 4.462 2.678 10.459 3.499 16.731 2.059 4.082-1.189 7.604-3.652 9.448-7.401 2.666-4.915 2.094-10.864 2.07-17.444.06-10.735.001-21.468.001-32.237z"></path></svg>',
  'javascript': '<svg viewBox="0 0 128 128" fill="currentColor"><path d="M2 1v125h125V1H2zm66.119 106.513c-1.845 3.749-5.367 6.212-9.448 7.401-6.271 1.44-12.269.619-16.731-2.059-2.986-1.832-5.318-4.652-6.901-7.901l9.52-5.83c.083.035.333.487.667 1.071 1.214 2.034 2.261 3.474 4.319 4.485 2.022.69 6.461 1.131 8.175-2.427 1.047-1.81.714-7.628.714-14.065C58.433 78.073 58.48 68 58.48 58h11.709c0 11 .06 21.418 0 32.152.025 6.58.596 12.446-2.07 17.361zm48.574-3.308c-4.07 13.922-26.762 14.374-35.83 5.176-1.916-2.165-3.117-3.296-4.26-5.795 4.819-2.772 4.819-2.772 9.508-5.485 2.547 3.915 4.902 6.068 9.139 6.949 5.748.702 11.531-1.273 10.234-7.378-1.333-4.986-11.77-6.199-18.873-11.531-7.211-4.843-8.901-16.611-2.975-23.335 1.975-2.487 5.343-4.343 8.877-5.235l3.688-.477c7.081-.143 11.507 1.727 14.756 5.355.904.916 1.642 1.904 3.022 4.045-3.772 2.404-3.76 2.381-9.163 5.879-1.154-2.486-3.069-4.046-5.093-4.724-3.142-.952-7.104.083-7.926 3.403-.285 1.023-.226 1.975.227 3.665 1.273 2.903 5.545 4.165 9.377 5.926 11.031 4.474 14.756 9.271 15.672 14.981.882 4.916-.213 8.105-.38 8.581z"></path></svg>',
  'http': globe,
  'prql': `<svg width="94" height="94" viewBox="0 0 94 94" fill="currentColor">
                  <path fill-rule="evenodd" clip-rule="evenodd" d="M42 67.763C31.7016 66.7705 24 62.7763 24 58V74C24 79.3959 33.8294 83.7936 46.1279 83.9929L37.9067 74.5H42V67.763ZM47.8721 83.9929L56.0933 74.5H52V67.763C62.2911 66.7712 69.9892 62.7819 70 58.0101V73.9899L70 74C70 79.3959 60.1706 83.7936 47.8721 83.9929Z"/>
                  <path fill-rule="evenodd" clip-rule="evenodd" d="M42 48.763C31.7016 47.7705 24 43.7763 24 39V55C24 59.7763 31.7016 63.7705 42 64.763V48.763ZM52 64.763V48.763C62.2911 47.7712 69.9892 43.7819 70 39.0101V54.9899L70 55C70 59.7763 62.2984 63.7705 52 64.763Z"/>
                  <path fill-rule="evenodd" clip-rule="evenodd" d="M47 10C34.2975 10 24 14.4772 24 20V36C24 40.7763 31.7016 44.7705 42 45.763V25.7363C35.0513 24.9827 30 22.6996 30 20C30 16.6863 37.6112 14 47 14C56.3888 14 64 16.6863 64 20C64 22.6996 58.9487 24.9827 52 25.7363V45.763C62.2984 44.7705 70 40.7763 70 36L70 35.9899V20.0101L70 20C70 14.4772 59.7025 10 47 10Z"/>
                </svg>`,
  'prql-color': `<svg width="94" height="94" viewBox="0 0 94 94" fill="none">
                  <path fill-rule="evenodd" clip-rule="evenodd" d="M42 67.763C31.7016 66.7705 24 62.7763 24 58V74C24 79.3959 33.8294 83.7936 46.1279 83.9929L37.9067 74.5H42V67.763ZM47.8721 83.9929L56.0933 74.5H52V67.763C62.2911 66.7712 69.9892 62.7819 70 58.0101V73.9899L70 74C70 79.3959 60.1706 83.7936 47.8721 83.9929Z" fill="#4F80E1"/>
                  <path fill-rule="evenodd" clip-rule="evenodd" d="M42 48.763C31.7016 47.7705 24 43.7763 24 39V55C24 59.7763 31.7016 63.7705 42 64.763V48.763ZM52 64.763V48.763C62.2911 47.7712 69.9892 43.7819 70 39.0101V54.9899L70 55C70 59.7763 62.2984 63.7705 52 64.763Z" fill="#CA4A36"/>
                  <path fill-rule="evenodd" clip-rule="evenodd" d="M47 10C34.2975 10 24 14.4772 24 20V36C24 40.7763 31.7016 44.7705 42 45.763V25.7363C35.0513 24.9827 30 22.6996 30 20C30 16.6863 37.6112 14 47 14C56.3888 14 64 16.6863 64 20C64 22.6996 58.9487 24.9827 52 25.7363V45.763C62.2984 44.7705 70 40.7763 70 36L70 35.9899V20.0101L70 20C70 14.4772 59.7025 10 47 10Z" fill="#DFB13C"/>
                </svg>`,
  'python-color': '<svg viewBox="0 0 128 128"><linearGradient id="python-original-a" gradientUnits="userSpaceOnUse" x1="70.252" y1="1237.476" x2="170.659" y2="1151.089" gradientTransform="matrix(.563 0 0 -.568 -29.215 707.817)"><stop offset="0" stop-color="#5A9FD4"></stop><stop offset="1" stop-color="#306998"></stop></linearGradient><linearGradient id="python-original-b" gradientUnits="userSpaceOnUse" x1="209.474" y1="1098.811" x2="173.62" y2="1149.537" gradientTransform="matrix(.563 0 0 -.568 -29.215 707.817)"><stop offset="0" stop-color="#FFD43B"></stop><stop offset="1" stop-color="#FFE873"></stop></linearGradient><path fill="url(#python-original-a)" d="M63.391 1.988c-4.222.02-8.252.379-11.8 1.007-10.45 1.846-12.346 5.71-12.346 12.837v9.411h24.693v3.137H29.977c-7.176 0-13.46 4.313-15.426 12.521-2.268 9.405-2.368 15.275 0 25.096 1.755 7.311 5.947 12.519 13.124 12.519h8.491V67.234c0-8.151 7.051-15.34 15.426-15.34h24.665c6.866 0 12.346-5.654 12.346-12.548V15.833c0-6.693-5.646-11.72-12.346-12.837-4.244-.706-8.645-1.027-12.866-1.008zM50.037 9.557c2.55 0 4.634 2.117 4.634 4.721 0 2.593-2.083 4.69-4.634 4.69-2.56 0-4.633-2.097-4.633-4.69-.001-2.604 2.073-4.721 4.633-4.721z" transform="translate(0 10.26)"></path><path fill="url(#python-original-b)" d="M91.682 28.38v10.966c0 8.5-7.208 15.655-15.426 15.655H51.591c-6.756 0-12.346 5.783-12.346 12.549v23.515c0 6.691 5.818 10.628 12.346 12.547 7.816 2.297 15.312 2.713 24.665 0 6.216-1.801 12.346-5.423 12.346-12.547v-9.412H63.938v-3.138h37.012c7.176 0 9.852-5.005 12.348-12.519 2.578-7.735 2.467-15.174 0-25.096-1.774-7.145-5.161-12.521-12.348-12.521h-9.268zM77.809 87.927c2.561 0 4.634 2.097 4.634 4.692 0 2.602-2.074 4.719-4.634 4.719-2.55 0-4.633-2.117-4.633-4.719 0-2.595 2.083-4.692 4.633-4.692z" transform="translate(0 10.26)"></path><radialGradient id="python-original-c" cx="1825.678" cy="444.45" r="26.743" gradientTransform="matrix(0 -.24 -1.055 0 532.979 557.576)" gradientUnits="userSpaceOnUse"><stop offset="0" stop-color="#B8B8B8" stop-opacity=".498"></stop><stop offset="1" stop-color="#7F7F7F" stop-opacity="0"></stop></radialGradient><path opacity=".444" fill="url(#python-original-c)" d="M97.309 119.597c0 3.543-14.816 6.416-33.091 6.416-18.276 0-33.092-2.873-33.092-6.416 0-3.544 14.815-6.417 33.092-6.417 18.275 0 33.091 2.872 33.091 6.417z"></path></svg>',
  'python': '<svg viewBox="0 0 128 128" fill="currentColor"><path d="M49.33 62h29.159C86.606 62 93 55.132 93 46.981V19.183c0-7.912-6.632-13.856-14.555-15.176-5.014-.835-10.195-1.215-15.187-1.191-4.99.023-9.612.448-13.805 1.191C37.098 6.188 35 10.758 35 19.183V30h29v4H23.776c-8.484 0-15.914 5.108-18.237 14.811-2.681 11.12-2.8 17.919 0 29.53C7.614 86.983 12.569 93 21.054 93H31V79.952C31 70.315 39.428 62 49.33 62zm-1.838-39.11c-3.026 0-5.478-2.479-5.478-5.545 0-3.079 2.451-5.581 5.478-5.581 3.015 0 5.479 2.502 5.479 5.581-.001 3.066-2.465 5.545-5.479 5.545zm74.789 25.921C120.183 40.363 116.178 34 107.682 34H97v12.981C97 57.031 88.206 65 78.489 65H49.33C41.342 65 35 72.326 35 80.326v27.8c0 7.91 6.745 12.564 14.462 14.834 9.242 2.717 17.994 3.208 29.051 0C85.862 120.831 93 116.549 93 108.126V97H64v-4h43.682c8.484 0 11.647-5.776 14.599-14.66 3.047-9.145 2.916-17.799 0-29.529zm-41.955 55.606c3.027 0 5.479 2.479 5.479 5.547 0 3.076-2.451 5.579-5.479 5.579-3.015 0-5.478-2.502-5.478-5.579 0-3.068 2.463-5.547 5.478-5.547z"></path></svg>',
  'r-color': '<svg viewBox="0 0 128 128"><defs><linearGradient id="r-original-a" x1=".741" x2="590.86" y1="3.666" y2="593.79" gradientTransform="matrix(.2169 0 0 .14527 -.16 14.112)" gradientUnits="userSpaceOnUse"><stop stop-color="#cbced0" offset="0"></stop><stop stop-color="#84838b" offset="1"></stop></linearGradient><linearGradient id="r-original-b" x1="301.03" x2="703.07" y1="151.4" y2="553.44" gradientTransform="matrix(.17572 0 0 .17931 -.16 14.112)" gradientUnits="userSpaceOnUse"><stop stop-color="#276dc3" offset="0"></stop><stop stop-color="#165caa" offset="1"></stop></linearGradient></defs><path d="M64 100.38c-35.346 0-64-19.19-64-42.863 0-23.672 28.654-42.863 64-42.863s64 19.19 64 42.863c0 23.672-28.654 42.863-64 42.863zm9.796-68.967c-26.866 0-48.646 13.119-48.646 29.303 0 16.183 21.78 29.303 48.646 29.303s46.693-8.97 46.693-29.303c0-20.327-19.827-29.303-46.693-29.303z" fill="url(#r-original-a)" fill-rule="evenodd"></path><path d="M97.469 81.033s3.874 1.169 6.124 2.308c.78.395 2.132 1.183 3.106 2.219a8.388 8.388 0 011.42 2.04l15.266 25.74-24.674.01-11.537-21.666s-2.363-4.06-3.817-5.237c-1.213-.982-1.73-1.331-2.929-1.331h-5.862l.004 28.219-21.833.009V41.26h43.844s19.97.36 19.97 19.359c0 18.999-19.082 20.413-19.082 20.413zm-9.497-24.137l-13.218-.009-.006 12.258 13.224-.005s6.124-.019 6.124-6.235c0-6.34-6.124-6.009-6.124-6.009z" fill="url(#r-original-b)" fill-rule="evenodd"></path></svg>',
  'r': '<svg viewBox="0 0 128 128" fill="currentColor"><path d="M64 14.648c-35.346 0-64 19.19-64 42.863C0 78.275 22.046 95.589 51.316 99.53V86.699c-15.55-4.89-26.166-14.693-26.166-25.991 0-16.183 21.779-29.303 48.646-29.303 26.866 0 46.693 8.975 46.693 29.303 0 10.486-5.273 17.95-14.066 22.72 1.204.908 2.22 2.072 2.904 3.419l.388.655C121.025 79.772 128 69.189 128 57.51c0-23.672-28.654-42.863-64-42.863zm20.1 74.88c-2.612.257-5.322.41-8.114.462l.002 9.63a88.362 88.362 0 0012.474-2.492l-.501-.941c-.68-1.268-1.347-2.543-2.033-3.807a41.01 41.01 0 00-1.828-2.851z"></path><path d="M97.469 81.036s3.874 1.169 6.124 2.307c.78.396 2.132 1.184 3.106 2.22a8.388 8.388 0 011.42 2.04l15.266 25.74-24.674.01-11.537-21.666s-2.363-4.06-3.817-5.237c-1.213-.982-1.73-1.331-2.929-1.331h-5.862l.004 28.219-21.834.009V41.263h43.845s19.97.36 19.97 19.359S97.47 81.035 97.47 81.035zm-9.497-24.137l-13.218-.009-.006 12.257 13.224-.004s6.124-.019 6.124-6.235c0-6.34-6.124-6.01-6.124-6.01z" fill-rule="evenodd"></path></svg>',
  'sql-color': '<svg viewBox="0 0 24 24" width="24" height="24"><path fill="none" d="M0 0h24v24H0z"/><path fill="#2f43b3" d="M5 12.5c0 .313.461.858 1.53 1.393C7.914 14.585 9.877 15 12 15c2.123 0 4.086-.415 5.47-1.107 1.069-.535 1.53-1.08 1.53-1.393v-2.171C17.35 11.349 14.827 12 12 12s-5.35-.652-7-1.671V12.5zm14 2.829C17.35 16.349 14.827 17 12 17s-5.35-.652-7-1.671V17.5c0 .313.461.858 1.53 1.393C7.914 19.585 9.877 20 12 20c2.123 0 4.086-.415 5.47-1.107 1.069-.535 1.53-1.08 1.53-1.393v-2.171zM3 17.5v-10C3 5.015 7.03 3 12 3s9 2.015 9 4.5v10c0 2.485-4.03 4.5-9 4.5s-9-2.015-9-4.5zm9-7.5c2.123 0 4.086-.415 5.47-1.107C18.539 8.358 19 7.813 19 7.5c0-.313-.461-.858-1.53-1.393C16.086 5.415 14.123 5 12 5c-2.123 0-4.086.415-5.47 1.107C5.461 6.642 5 7.187 5 7.5c0 .313.461.858 1.53 1.393C7.914 9.585 9.877 10 12 10z"/></svg>',
  'sql': '<svg viewBox="0 0 24 24" width="24" height="24"><path fill="none" d="M0 0h24v24H0z"/><path fill="#777" d="M5 12.5c0 .313.461.858 1.53 1.393C7.914 14.585 9.877 15 12 15c2.123 0 4.086-.415 5.47-1.107 1.069-.535 1.53-1.08 1.53-1.393v-2.171C17.35 11.349 14.827 12 12 12s-5.35-.652-7-1.671V12.5zm14 2.829C17.35 16.349 14.827 17 12 17s-5.35-.652-7-1.671V17.5c0 .313.461.858 1.53 1.393C7.914 19.585 9.877 20 12 20c2.123 0 4.086-.415 5.47-1.107 1.069-.535 1.53-1.08 1.53-1.393v-2.171zM3 17.5v-10C3 5.015 7.03 3 12 3s9 2.015 9 4.5v10c0 2.485-4.03 4.5-9 4.5s-9-2.015-9-4.5zm9-7.5c2.123 0 4.086-.415 5.47-1.107C18.539 8.358 19 7.813 19 7.5c0-.313-.461-.858-1.53-1.393C16.086 5.415 14.123 5 12 5c-2.123 0-4.086.415-5.47 1.107C5.461 6.642 5 7.187 5 7.5c0 .313.461.858 1.53 1.393C7.914 9.585 9.877 10 12 10z"/></svg>',
  'tailwind-color': '<svg viewBox="0 0 128 128"><path d="M64.004 25.602c-17.067 0-27.73 8.53-32 25.597 6.398-8.531 13.867-11.73 22.398-9.597 4.871 1.214 8.352 4.746 12.207 8.66C72.883 56.629 80.145 64 96.004 64c17.066 0 27.73-8.531 32-25.602-6.399 8.536-13.867 11.735-22.399 9.602-4.87-1.215-8.347-4.746-12.207-8.66-6.27-6.367-13.53-13.738-29.394-13.738zM32.004 64c-17.066 0-27.73 8.531-32 25.602C6.402 81.066 13.87 77.867 22.402 80c4.871 1.215 8.352 4.746 12.207 8.66 6.274 6.367 13.536 13.738 29.395 13.738 17.066 0 27.73-8.53 32-25.597-6.399 8.531-13.867 11.73-22.399 9.597-4.87-1.214-8.347-4.746-12.207-8.66C55.128 71.371 47.868 64 32.004 64zm0 0" fill="#38b2ac"></path></svg>',
  'tailwind': '<svg viewBox="0 0 128 128"><path d="M64.004 25.602c-17.067 0-27.73 8.53-32 25.597 6.398-8.531 13.867-11.73 22.398-9.597 4.871 1.214 8.352 4.746 12.207 8.66C72.883 56.629 80.145 64 96.004 64c17.066 0 27.73-8.531 32-25.602-6.399 8.536-13.867 11.735-22.399 9.602-4.87-1.215-8.347-4.746-12.207-8.66-6.27-6.367-13.53-13.738-29.394-13.738zM32.004 64c-17.066 0-27.73 8.531-32 25.602C6.402 81.066 13.87 77.867 22.402 80c4.871 1.215 8.352 4.746 12.207 8.66 6.274 6.367 13.536 13.738 29.395 13.738 17.066 0 27.73-8.53 32-25.597-6.399 8.531-13.867 11.73-22.399 9.597-4.87-1.214-8.347-4.746-12.207-8.66C55.128 71.371 47.868 64 32.004 64zm0 0" fill="#777"></path></svg>',
  'unknown': questionSquare,
}
