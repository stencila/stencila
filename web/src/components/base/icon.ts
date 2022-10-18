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
  if (icons[name] === undefined) {
    console.warn(`No icon with name "${name}"`)
  }
  const svg = encodeURIComponent(icons[name] ?? icons.circle)
  return `data:image/svg+xml,${svg}`
}

const braces = require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/braces.svg')
const calculator = require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/calculator.svg')
const questionSquare = require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/question-square.svg')

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
  'braces-asterisk': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/braces-asterisk.svg'),
  'broadcast-pin': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/broadcast-pin.svg'),
  'brush': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/brush.svg'),
  'bug': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/bug.svg'),
  'check-circle': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/check-circle.svg'),
  'chevron-down': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/chevron-down.svg'),
  'chevron-right': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/chevron-right.svg'),
  'chevron-left': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/chevron-left.svg'),
  'circle': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/circle.svg'),
  'clock': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/clock.svg'),
  'code-square': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/code-square.svg'),
  'code': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/code.svg'),
  'dash-circle': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/dash-circle.svg'),
  'dash': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/dash.svg'),
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
  'hourglass': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/hourglass.svg'),
  'house': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/house.svg'),
  'info-circle': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/info-circle.svg'),
  'keyboard': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/keyboard.svg'),
  'lightning-fill': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/lightning-fill.svg'),
  'list': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/list.svg'),
  'list-nested': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/list-nested.svg'),
  'map': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/map.svg'),
  'magic': require('bundle-text:@shoelace-style/shoelace/dist/assets/icons/magic.svg'),
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

  // Language icons, mostly from https://devicon.dev/ but some from project sites and Remix Icon
  'bash-color': '<svg viewBox="0 0 128 128"><path fill="none" d="M-143.76 4.24h119.53v119.53h-119.53z"></path><path fill="#293138" d="M109.01 28.64L71.28 6.24c-2.25-1.33-4.77-2-7.28-2s-5.03.67-7.28 2.01l-37.74 22.4c-4.5 2.67-7.28 7.61-7.28 12.96v44.8c0 5.35 2.77 10.29 7.28 12.96l37.73 22.4c2.25 1.34 4.76 2 7.28 2 2.51 0 5.03-.67 7.28-2l37.74-22.4c4.5-2.67 7.28-7.62 7.28-12.96V41.6c0-5.34-2.77-10.29-7.28-12.96zM79.79 98.59l.06 3.22c0 .39-.25.83-.55.99l-1.91 1.1c-.3.15-.56-.03-.56-.42l-.03-3.17c-1.63.68-3.29.84-4.34.42-.2-.08-.29-.37-.21-.71l.69-2.91c.06-.23.18-.46.34-.6.06-.06.12-.1.18-.13.11-.06.22-.07.31-.03 1.14.38 2.59.2 3.99-.5 1.78-.9 2.97-2.72 2.95-4.52-.02-1.64-.9-2.31-3.05-2.33-2.74.01-5.3-.53-5.34-4.57-.03-3.32 1.69-6.78 4.43-8.96l-.03-3.25c0-.4.24-.84.55-1l1.85-1.18c.3-.15.56.04.56.43l.03 3.25c1.36-.54 2.54-.69 3.61-.44.23.06.34.38.24.75l-.72 2.88c-.06.22-.18.44-.33.58a.77.77 0 01-.19.14c-.1.05-.19.06-.28.05-.49-.11-1.65-.36-3.48.56-1.92.97-2.59 2.64-2.58 3.88.02 1.48.77 1.93 3.39 1.97 3.49.06 4.99 1.58 5.03 5.09.05 3.44-1.79 7.15-4.61 9.41zm19.78-5.41c0 .3-.04.58-.29.72l-9.54 5.8c-.25.15-.45.02-.45-.28v-2.46c0-.3.18-.46.43-.61l9.4-5.62c.25-.15.45-.02.45.28v2.17zm6.56-55.09l-35.7 22.05c-4.45 2.6-7.73 5.52-7.74 10.89v43.99c0 3.21 1.3 5.29 3.29 5.9-.65.11-1.32.19-1.98.19-2.09 0-4.15-.57-5.96-1.64l-37.73-22.4c-3.69-2.19-5.98-6.28-5.98-10.67V41.6c0-4.39 2.29-8.48 5.98-10.67l37.74-22.4c1.81-1.07 3.87-1.64 5.96-1.64s4.15.57 5.96 1.64l37.74 22.4c3.11 1.85 5.21 5.04 5.8 8.63-1.27-2.67-4.09-3.39-7.38-1.47z"></path></svg>',
  'bash': '<svg viewBox="0 0 128 128"><path fill="none" d="M-143.76 4.24h119.53v119.53h-119.53z"></path><path fill="#293138" d="M109.01 28.64L71.28 6.24c-2.25-1.33-4.77-2-7.28-2s-5.03.67-7.28 2.01l-37.74 22.4c-4.5 2.67-7.28 7.61-7.28 12.96v44.8c0 5.35 2.77 10.29 7.28 12.96l37.73 22.4c2.25 1.34 4.76 2 7.28 2 2.51 0 5.03-.67 7.28-2l37.74-22.4c4.5-2.67 7.28-7.62 7.28-12.96V41.6c0-5.34-2.77-10.29-7.28-12.96zM79.79 98.59l.06 3.22c0 .39-.25.83-.55.99l-1.91 1.1c-.3.15-.56-.03-.56-.42l-.03-3.17c-1.63.68-3.29.84-4.34.42-.2-.08-.29-.37-.21-.71l.69-2.91c.06-.23.18-.46.34-.6.06-.06.12-.1.18-.13.11-.06.22-.07.31-.03 1.14.38 2.59.2 3.99-.5 1.78-.9 2.97-2.72 2.95-4.52-.02-1.64-.9-2.31-3.05-2.33-2.74.01-5.3-.53-5.34-4.57-.03-3.32 1.69-6.78 4.43-8.96l-.03-3.25c0-.4.24-.84.55-1l1.85-1.18c.3-.15.56.04.56.43l.03 3.25c1.36-.54 2.54-.69 3.61-.44.23.06.34.38.24.75l-.72 2.88c-.06.22-.18.44-.33.58a.77.77 0 01-.19.14c-.1.05-.19.06-.28.05-.49-.11-1.65-.36-3.48.56-1.92.97-2.59 2.64-2.58 3.88.02 1.48.77 1.93 3.39 1.97 3.49.06 4.99 1.58 5.03 5.09.05 3.44-1.79 7.15-4.61 9.41zm19.78-5.41c0 .3-.04.58-.29.72l-9.54 5.8c-.25.15-.45.02-.45-.28v-2.46c0-.3.18-.46.43-.61l9.4-5.62c.25-.15.45-.02.45.28v2.17zm6.56-55.09l-35.7 22.05c-4.45 2.6-7.73 5.52-7.74 10.89v43.99c0 3.21 1.3 5.29 3.29 5.9-.65.11-1.32.19-1.98.19-2.09 0-4.15-.57-5.96-1.64l-37.73-22.4c-3.69-2.19-5.98-6.28-5.98-10.67V41.6c0-4.39 2.29-8.48 5.98-10.67l37.74-22.4c1.81-1.07 3.87-1.64 5.96-1.64s4.15.57 5.96 1.64l37.74 22.4c3.11 1.85 5.21 5.04 5.8 8.63-1.27-2.67-4.09-3.39-7.38-1.47z"></path></svg>',
  'calc': calculator,
  'calc-color': calculator,
  'javascript-color': '<svg viewBox="0 0 128 128"><path fill="#F0DB4F" d="M1.408 1.408h125.184v125.185H1.408z"></path><path fill="#323330" d="M116.347 96.736c-.917-5.711-4.641-10.508-15.672-14.981-3.832-1.761-8.104-3.022-9.377-5.926-.452-1.69-.512-2.642-.226-3.665.821-3.32 4.784-4.355 7.925-3.403 2.023.678 3.938 2.237 5.093 4.724 5.402-3.498 5.391-3.475 9.163-5.879-1.381-2.141-2.118-3.129-3.022-4.045-3.249-3.629-7.676-5.498-14.756-5.355l-3.688.477c-3.534.893-6.902 2.748-8.877 5.235-5.926 6.724-4.236 18.492 2.975 23.335 7.104 5.332 17.54 6.545 18.873 11.531 1.297 6.104-4.486 8.08-10.234 7.378-4.236-.881-6.592-3.034-9.139-6.949-4.688 2.713-4.688 2.713-9.508 5.485 1.143 2.499 2.344 3.63 4.26 5.795 9.068 9.198 31.76 8.746 35.83-5.176.165-.478 1.261-3.666.38-8.581zM69.462 58.943H57.753l-.048 30.272c0 6.438.333 12.34-.714 14.149-1.713 3.558-6.152 3.117-8.175 2.427-2.059-1.012-3.106-2.451-4.319-4.485-.333-.584-.583-1.036-.667-1.071l-9.52 5.83c1.583 3.249 3.915 6.069 6.902 7.901 4.462 2.678 10.459 3.499 16.731 2.059 4.082-1.189 7.604-3.652 9.448-7.401 2.666-4.915 2.094-10.864 2.07-17.444.06-10.735.001-21.468.001-32.237z"></path></svg>',
  'javascript': '<svg viewBox="0 0 128 128" fill="#777"><path d="M2 1v125h125V1H2zm66.119 106.513c-1.845 3.749-5.367 6.212-9.448 7.401-6.271 1.44-12.269.619-16.731-2.059-2.986-1.832-5.318-4.652-6.901-7.901l9.52-5.83c.083.035.333.487.667 1.071 1.214 2.034 2.261 3.474 4.319 4.485 2.022.69 6.461 1.131 8.175-2.427 1.047-1.81.714-7.628.714-14.065C58.433 78.073 58.48 68 58.48 58h11.709c0 11 .06 21.418 0 32.152.025 6.58.596 12.446-2.07 17.361zm48.574-3.308c-4.07 13.922-26.762 14.374-35.83 5.176-1.916-2.165-3.117-3.296-4.26-5.795 4.819-2.772 4.819-2.772 9.508-5.485 2.547 3.915 4.902 6.068 9.139 6.949 5.748.702 11.531-1.273 10.234-7.378-1.333-4.986-11.77-6.199-18.873-11.531-7.211-4.843-8.901-16.611-2.975-23.335 1.975-2.487 5.343-4.343 8.877-5.235l3.688-.477c7.081-.143 11.507 1.727 14.756 5.355.904.916 1.642 1.904 3.022 4.045-3.772 2.404-3.76 2.381-9.163 5.879-1.154-2.486-3.069-4.046-5.093-4.724-3.142-.952-7.104.083-7.926 3.403-.285 1.023-.226 1.975.227 3.665 1.273 2.903 5.545 4.165 9.377 5.926 11.031 4.474 14.756 9.271 15.672 14.981.882 4.916-.213 8.105-.38 8.581z"></path></svg>',
  'json-color': braces,
  'json': braces,
  'json5-color': braces,
  'json5': braces,
  'prql': `<svg width="94" height="94" viewBox="0 0 94 94" fill="none">
                  <path fill-rule="evenodd" clip-rule="evenodd" d="M42 67.763C31.7016 66.7705 24 62.7763 24 58V74C24 79.3959 33.8294 83.7936 46.1279 83.9929L37.9067 74.5H42V67.763ZM47.8721 83.9929L56.0933 74.5H52V67.763C62.2911 66.7712 69.9892 62.7819 70 58.0101V73.9899L70 74C70 79.3959 60.1706 83.7936 47.8721 83.9929Z" fill="#777"/>
                  <path fill-rule="evenodd" clip-rule="evenodd" d="M42 48.763C31.7016 47.7705 24 43.7763 24 39V55C24 59.7763 31.7016 63.7705 42 64.763V48.763ZM52 64.763V48.763C62.2911 47.7712 69.9892 43.7819 70 39.0101V54.9899L70 55C70 59.7763 62.2984 63.7705 52 64.763Z" fill="#777"/>
                  <path fill-rule="evenodd" clip-rule="evenodd" d="M47 10C34.2975 10 24 14.4772 24 20V36C24 40.7763 31.7016 44.7705 42 45.763V25.7363C35.0513 24.9827 30 22.6996 30 20C30 16.6863 37.6112 14 47 14C56.3888 14 64 16.6863 64 20C64 22.6996 58.9487 24.9827 52 25.7363V45.763C62.2984 44.7705 70 40.7763 70 36L70 35.9899V20.0101L70 20C70 14.4772 59.7025 10 47 10Z" fill="#777"/>
                </svg>`,
  'prql-color': `<svg width="94" height="94" viewBox="0 0 94 94" fill="none">
                  <path fill-rule="evenodd" clip-rule="evenodd" d="M42 67.763C31.7016 66.7705 24 62.7763 24 58V74C24 79.3959 33.8294 83.7936 46.1279 83.9929L37.9067 74.5H42V67.763ZM47.8721 83.9929L56.0933 74.5H52V67.763C62.2911 66.7712 69.9892 62.7819 70 58.0101V73.9899L70 74C70 79.3959 60.1706 83.7936 47.8721 83.9929Z" fill="#4F80E1"/>
                  <path fill-rule="evenodd" clip-rule="evenodd" d="M42 48.763C31.7016 47.7705 24 43.7763 24 39V55C24 59.7763 31.7016 63.7705 42 64.763V48.763ZM52 64.763V48.763C62.2911 47.7712 69.9892 43.7819 70 39.0101V54.9899L70 55C70 59.7763 62.2984 63.7705 52 64.763Z" fill="#CA4A36"/>
                  <path fill-rule="evenodd" clip-rule="evenodd" d="M47 10C34.2975 10 24 14.4772 24 20V36C24 40.7763 31.7016 44.7705 42 45.763V25.7363C35.0513 24.9827 30 22.6996 30 20C30 16.6863 37.6112 14 47 14C56.3888 14 64 16.6863 64 20C64 22.6996 58.9487 24.9827 52 25.7363V45.763C62.2984 44.7705 70 40.7763 70 36L70 35.9899V20.0101L70 20C70 14.4772 59.7025 10 47 10Z" fill="#DFB13C"/>
                </svg>`,
  'python-color': '<svg viewBox="0 0 128 128"><linearGradient id="python-original-a" gradientUnits="userSpaceOnUse" x1="70.252" y1="1237.476" x2="170.659" y2="1151.089" gradientTransform="matrix(.563 0 0 -.568 -29.215 707.817)"><stop offset="0" stop-color="#5A9FD4"></stop><stop offset="1" stop-color="#306998"></stop></linearGradient><linearGradient id="python-original-b" gradientUnits="userSpaceOnUse" x1="209.474" y1="1098.811" x2="173.62" y2="1149.537" gradientTransform="matrix(.563 0 0 -.568 -29.215 707.817)"><stop offset="0" stop-color="#FFD43B"></stop><stop offset="1" stop-color="#FFE873"></stop></linearGradient><path fill="url(#python-original-a)" d="M63.391 1.988c-4.222.02-8.252.379-11.8 1.007-10.45 1.846-12.346 5.71-12.346 12.837v9.411h24.693v3.137H29.977c-7.176 0-13.46 4.313-15.426 12.521-2.268 9.405-2.368 15.275 0 25.096 1.755 7.311 5.947 12.519 13.124 12.519h8.491V67.234c0-8.151 7.051-15.34 15.426-15.34h24.665c6.866 0 12.346-5.654 12.346-12.548V15.833c0-6.693-5.646-11.72-12.346-12.837-4.244-.706-8.645-1.027-12.866-1.008zM50.037 9.557c2.55 0 4.634 2.117 4.634 4.721 0 2.593-2.083 4.69-4.634 4.69-2.56 0-4.633-2.097-4.633-4.69-.001-2.604 2.073-4.721 4.633-4.721z" transform="translate(0 10.26)"></path><path fill="url(#python-original-b)" d="M91.682 28.38v10.966c0 8.5-7.208 15.655-15.426 15.655H51.591c-6.756 0-12.346 5.783-12.346 12.549v23.515c0 6.691 5.818 10.628 12.346 12.547 7.816 2.297 15.312 2.713 24.665 0 6.216-1.801 12.346-5.423 12.346-12.547v-9.412H63.938v-3.138h37.012c7.176 0 9.852-5.005 12.348-12.519 2.578-7.735 2.467-15.174 0-25.096-1.774-7.145-5.161-12.521-12.348-12.521h-9.268zM77.809 87.927c2.561 0 4.634 2.097 4.634 4.692 0 2.602-2.074 4.719-4.634 4.719-2.55 0-4.633-2.117-4.633-4.719 0-2.595 2.083-4.692 4.633-4.692z" transform="translate(0 10.26)"></path><radialGradient id="python-original-c" cx="1825.678" cy="444.45" r="26.743" gradientTransform="matrix(0 -.24 -1.055 0 532.979 557.576)" gradientUnits="userSpaceOnUse"><stop offset="0" stop-color="#B8B8B8" stop-opacity=".498"></stop><stop offset="1" stop-color="#7F7F7F" stop-opacity="0"></stop></radialGradient><path opacity=".444" fill="url(#python-original-c)" d="M97.309 119.597c0 3.543-14.816 6.416-33.091 6.416-18.276 0-33.092-2.873-33.092-6.416 0-3.544 14.815-6.417 33.092-6.417 18.275 0 33.091 2.872 33.091 6.417z"></path></svg>',
  'python': '<svg viewBox="0 0 128 128" fill="#777"><path d="M49.33 62h29.159C86.606 62 93 55.132 93 46.981V19.183c0-7.912-6.632-13.856-14.555-15.176-5.014-.835-10.195-1.215-15.187-1.191-4.99.023-9.612.448-13.805 1.191C37.098 6.188 35 10.758 35 19.183V30h29v4H23.776c-8.484 0-15.914 5.108-18.237 14.811-2.681 11.12-2.8 17.919 0 29.53C7.614 86.983 12.569 93 21.054 93H31V79.952C31 70.315 39.428 62 49.33 62zm-1.838-39.11c-3.026 0-5.478-2.479-5.478-5.545 0-3.079 2.451-5.581 5.478-5.581 3.015 0 5.479 2.502 5.479 5.581-.001 3.066-2.465 5.545-5.479 5.545zm74.789 25.921C120.183 40.363 116.178 34 107.682 34H97v12.981C97 57.031 88.206 65 78.489 65H49.33C41.342 65 35 72.326 35 80.326v27.8c0 7.91 6.745 12.564 14.462 14.834 9.242 2.717 17.994 3.208 29.051 0C85.862 120.831 93 116.549 93 108.126V97H64v-4h43.682c8.484 0 11.647-5.776 14.599-14.66 3.047-9.145 2.916-17.799 0-29.529zm-41.955 55.606c3.027 0 5.479 2.479 5.479 5.547 0 3.076-2.451 5.579-5.479 5.579-3.015 0-5.478-2.502-5.478-5.579 0-3.068 2.463-5.547 5.478-5.547z"></path></svg>',
  'r-color': '<svg viewBox="0 0 128 128"><defs><linearGradient id="r-original-a" x1=".741" x2="590.86" y1="3.666" y2="593.79" gradientTransform="matrix(.2169 0 0 .14527 -.16 14.112)" gradientUnits="userSpaceOnUse"><stop stop-color="#cbced0" offset="0"></stop><stop stop-color="#84838b" offset="1"></stop></linearGradient><linearGradient id="r-original-b" x1="301.03" x2="703.07" y1="151.4" y2="553.44" gradientTransform="matrix(.17572 0 0 .17931 -.16 14.112)" gradientUnits="userSpaceOnUse"><stop stop-color="#276dc3" offset="0"></stop><stop stop-color="#165caa" offset="1"></stop></linearGradient></defs><path d="M64 100.38c-35.346 0-64-19.19-64-42.863 0-23.672 28.654-42.863 64-42.863s64 19.19 64 42.863c0 23.672-28.654 42.863-64 42.863zm9.796-68.967c-26.866 0-48.646 13.119-48.646 29.303 0 16.183 21.78 29.303 48.646 29.303s46.693-8.97 46.693-29.303c0-20.327-19.827-29.303-46.693-29.303z" fill="url(#r-original-a)" fill-rule="evenodd"></path><path d="M97.469 81.033s3.874 1.169 6.124 2.308c.78.395 2.132 1.183 3.106 2.219a8.388 8.388 0 011.42 2.04l15.266 25.74-24.674.01-11.537-21.666s-2.363-4.06-3.817-5.237c-1.213-.982-1.73-1.331-2.929-1.331h-5.862l.004 28.219-21.833.009V41.26h43.844s19.97.36 19.97 19.359c0 18.999-19.082 20.413-19.082 20.413zm-9.497-24.137l-13.218-.009-.006 12.258 13.224-.005s6.124-.019 6.124-6.235c0-6.34-6.124-6.009-6.124-6.009z" fill="url(#r-original-b)" fill-rule="evenodd"></path></svg>',
  'r': '<svg viewBox="0 0 128 128" fill="#777"><path d="M64 14.648c-35.346 0-64 19.19-64 42.863C0 78.275 22.046 95.589 51.316 99.53V86.699c-15.55-4.89-26.166-14.693-26.166-25.991 0-16.183 21.779-29.303 48.646-29.303 26.866 0 46.693 8.975 46.693 29.303 0 10.486-5.273 17.95-14.066 22.72 1.204.908 2.22 2.072 2.904 3.419l.388.655C121.025 79.772 128 69.189 128 57.51c0-23.672-28.654-42.863-64-42.863zm20.1 74.88c-2.612.257-5.322.41-8.114.462l.002 9.63a88.362 88.362 0 0012.474-2.492l-.501-.941c-.68-1.268-1.347-2.543-2.033-3.807a41.01 41.01 0 00-1.828-2.851z"></path><path d="M97.469 81.036s3.874 1.169 6.124 2.307c.78.396 2.132 1.184 3.106 2.22a8.388 8.388 0 011.42 2.04l15.266 25.74-24.674.01-11.537-21.666s-2.363-4.06-3.817-5.237c-1.213-.982-1.73-1.331-2.929-1.331h-5.862l.004 28.219-21.834.009V41.263h43.845s19.97.36 19.97 19.359S97.47 81.035 97.47 81.035zm-9.497-24.137l-13.218-.009-.006 12.257 13.224-.004s6.124-.019 6.124-6.235c0-6.34-6.124-6.01-6.124-6.01z" fill-rule="evenodd"></path></svg>',
  'sql-color': '<svg viewBox="0 0 24 24" width="24" height="24"><path fill="none" d="M0 0h24v24H0z"/><path fill="#2f43b3" d="M5 12.5c0 .313.461.858 1.53 1.393C7.914 14.585 9.877 15 12 15c2.123 0 4.086-.415 5.47-1.107 1.069-.535 1.53-1.08 1.53-1.393v-2.171C17.35 11.349 14.827 12 12 12s-5.35-.652-7-1.671V12.5zm14 2.829C17.35 16.349 14.827 17 12 17s-5.35-.652-7-1.671V17.5c0 .313.461.858 1.53 1.393C7.914 19.585 9.877 20 12 20c2.123 0 4.086-.415 5.47-1.107 1.069-.535 1.53-1.08 1.53-1.393v-2.171zM3 17.5v-10C3 5.015 7.03 3 12 3s9 2.015 9 4.5v10c0 2.485-4.03 4.5-9 4.5s-9-2.015-9-4.5zm9-7.5c2.123 0 4.086-.415 5.47-1.107C18.539 8.358 19 7.813 19 7.5c0-.313-.461-.858-1.53-1.393C16.086 5.415 14.123 5 12 5c-2.123 0-4.086.415-5.47 1.107C5.461 6.642 5 7.187 5 7.5c0 .313.461.858 1.53 1.393C7.914 9.585 9.877 10 12 10z"/></svg>',
  'sql': '<svg viewBox="0 0 24 24" width="24" height="24"><path fill="none" d="M0 0h24v24H0z"/><path fill="#777" d="M5 12.5c0 .313.461.858 1.53 1.393C7.914 14.585 9.877 15 12 15c2.123 0 4.086-.415 5.47-1.107 1.069-.535 1.53-1.08 1.53-1.393v-2.171C17.35 11.349 14.827 12 12 12s-5.35-.652-7-1.671V12.5zm14 2.829C17.35 16.349 14.827 17 12 17s-5.35-.652-7-1.671V17.5c0 .313.461.858 1.53 1.393C7.914 19.585 9.877 20 12 20c2.123 0 4.086-.415 5.47-1.107 1.069-.535 1.53-1.08 1.53-1.393v-2.171zM3 17.5v-10C3 5.015 7.03 3 12 3s9 2.015 9 4.5v10c0 2.485-4.03 4.5-9 4.5s-9-2.015-9-4.5zm9-7.5c2.123 0 4.086-.415 5.47-1.107C18.539 8.358 19 7.813 19 7.5c0-.313-.461-.858-1.53-1.393C16.086 5.415 14.123 5 12 5c-2.123 0-4.086.415-5.47 1.107C5.461 6.642 5 7.187 5 7.5c0 .313.461.858 1.53 1.393C7.914 9.585 9.877 10 12 10z"/></svg>',
  'tailwind-color': '<svg viewBox="0 0 128 128"><path d="M64.004 25.602c-17.067 0-27.73 8.53-32 25.597 6.398-8.531 13.867-11.73 22.398-9.597 4.871 1.214 8.352 4.746 12.207 8.66C72.883 56.629 80.145 64 96.004 64c17.066 0 27.73-8.531 32-25.602-6.399 8.536-13.867 11.735-22.399 9.602-4.87-1.215-8.347-4.746-12.207-8.66-6.27-6.367-13.53-13.738-29.394-13.738zM32.004 64c-17.066 0-27.73 8.531-32 25.602C6.402 81.066 13.87 77.867 22.402 80c4.871 1.215 8.352 4.746 12.207 8.66 6.274 6.367 13.536 13.738 29.395 13.738 17.066 0 27.73-8.53 32-25.597-6.399 8.531-13.867 11.73-22.399 9.597-4.87-1.214-8.347-4.746-12.207-8.66C55.128 71.371 47.868 64 32.004 64zm0 0" fill="#38b2ac"></path></svg>',
  'tailwind': '<svg viewBox="0 0 128 128"><path d="M64.004 25.602c-17.067 0-27.73 8.53-32 25.597 6.398-8.531 13.867-11.73 22.398-9.597 4.871 1.214 8.352 4.746 12.207 8.66C72.883 56.629 80.145 64 96.004 64c17.066 0 27.73-8.531 32-25.602-6.399 8.536-13.867 11.735-22.399 9.602-4.87-1.215-8.347-4.746-12.207-8.66-6.27-6.367-13.53-13.738-29.394-13.738zM32.004 64c-17.066 0-27.73 8.531-32 25.602C6.402 81.066 13.87 77.867 22.402 80c4.871 1.215 8.352 4.746 12.207 8.66 6.274 6.367 13.536 13.738 29.395 13.738 17.066 0 27.73-8.53 32-25.597-6.399 8.531-13.867 11.73-22.399 9.597-4.87-1.214-8.347-4.746-12.207-8.66C55.128 71.371 47.868 64 32.004 64zm0 0" fill="#777"></path></svg>',
  'unknown': questionSquare,
  'unknown-color': questionSquare
}
