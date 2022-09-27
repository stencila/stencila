import 'time-elements'
import { customElement, property } from 'lit/decorators'

import StencilaElement from '../utils/element'
import { html } from 'lit'

/**
 * A component to represent the `Timestamp` node type
 */
@customElement('stencila-timestamp')
export default class StencilaTimestamp extends StencilaElement {
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
    const millis = (() => {
      switch (this.timeUnit) {
        case 'Year':
          return this.value * 3.154e10
        case 'Month':
          return this.value * 2.628e9
        case 'Week':
          return this.value * 6.048e8
        case 'Day':
          return this.value * 8.64e7
        case 'Hour':
          return this.value * 3.6e6
        case 'Minute':
          return this.value * 6e4
        case 'Second':
          return this.value * 1e3
        case 'Millisecond':
          return this.value
        case 'Microsecond':
          return this.value / 1e3
        case 'Nanosecond':
          return this.value / 1e6
        case 'Picosecond':
          return this.value / 1e9
        case 'Femtosecond':
          return this.value / 1e12
        case 'Attosecond':
          return this.value / 1e15
      }
    })()

    const date = new Date(millis * 1000)
    const iso8601 = date.toISOString()

    return html`<relative-time datetime=${iso8601}></relative-time>`
  }
}
