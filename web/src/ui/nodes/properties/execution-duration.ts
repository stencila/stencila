import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'
import prettyMilliseconds from 'pretty-ms'

import { withTwind } from '../../../twind'

import './generic/simple'

/**
 * A component for displaying the `executionDuration` property of executable nodes
 */
@customElement('stencila-ui-node-execution-duration')
@withTwind()
export class UINodeExecutionDuration extends LitElement {
  @property({ type: Number })
  value?: number = undefined

  override render() {
    const value =
      this.value === undefined
        ? '-'
        : prettyMilliseconds(this.value, { separateMilliseconds: true })

    const tooltip = `Last execution took ${
      this.value === 0 ? 'less than a' : `${this.value}`
    } millisecond${this.value > 1 ? 's' : ''}`

    return html`
      <stencila-ui-node-simple-property
        icon-name="stopwatch"
        icon-library="default"
        tooltip-content=${tooltip}
      >
        ${value}
      </stencila-ui-node-simple-property>
    `
  }
}
