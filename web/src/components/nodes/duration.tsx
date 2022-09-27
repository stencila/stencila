import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import StencilaElement from '../utils/element'

/**
 * A component to represent the `Duration` node type
 */
@customElement('stencila-duration')
export default class StencilaDuration extends StencilaElement {
  @property({ type: Number })
  value: number

  @property()
  timeUnit:
    | 'Year'
    | 'Month'
    | 'Week'
    | 'Day'
    | 'Hour'
    | 'Minute'
    | 'Second'
    | 'Millisecond'
    | 'Microsecond'
    | 'Nanosecond'
    | 'Picosecond'
    | 'Femtosecond'
    | 'Attosecond' = 'Microsecond'

  render() {
    const [value, unit] = (() => {
      switch (this.timeUnit) {
        case 'Minute': {
          if (this.value > 1.44e3) {
            return [this.value / 1.44e3, 'Day']
          } else if (this.value > 6e1) {
            return [this.value / 6e1, 'Hour']
          } else {
            return [this.value, this.timeUnit]
          }
        }
        case 'Second': {
          if (this.value > 8.64e4) {
            return [this.value / 8.64e4, 'Day']
          } else if (this.value > 3.6e3) {
            return [this.value / 3.6e3, 'Hour']
          } else if (this.value > 6e1) {
            return [this.value / 6e1, 'Minute']
          } else {
            return [this.value, this.timeUnit]
          }
        }
        case 'Millisecond': {
          if (this.value > 8.64e7) {
            return [this.value / 8.64e7, 'Day']
          } else if (this.value > 3.6e6) {
            return [this.value / 3.6e6, 'Hour']
          } else if (this.value > 6e4) {
            return [this.value / 6e4, 'Minute']
          } else if (this.value > 1e3) {
            return [this.value / 1e3, 'Second']
          } else {
            return [this.value, this.timeUnit]
          }
        }
        case 'Microsecond': {
          if (this.value > 3.6e9) {
            return [this.value / 3.6e9, 'Hour']
          } else if (this.value > 6e7) {
            return [this.value / 6e7, 'Minute']
          } else if (this.value > 1e6) {
            return [this.value / 1e6, 'Second']
          } else if (this.value > 1e3) {
            return [this.value / 1e3, 'Millisecond']
          } else {
            return [this.value, this.timeUnit]
          }
        }
        default:
          return [this.value, this.timeUnit]
      }
    })()

    const rounded = Math.round(value)

    const symbol = (() => {
      switch (unit) {
        case 'Year':
          return 'yr'
        case 'Month':
          return 'mo'
        case 'Week':
          return 'wk'
        case 'Day':
          return 'd'
        case 'Hour':
          return 'h'
        case 'Minute':
          return 'min'
        case 'Second':
          return 's'
        case 'Millisecond':
          return 'ms'
        case 'Microsecond':
          return 'µs'
        case 'Nanosecond':
          return 'ns'
        case 'Picosecond':
          return 'ps'
        case 'Femtosecond':
          return 'fs'
        case 'Attosecond':
          return 'as'
      }
    })()

    return html`<span>${rounded}${symbol}</span>`
  }
}
