import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../twind'

import './generic/simple'

/**
 * A component for displaying the `executionCount` property of executable nodes
 */
@customElement('stencila-ui-node-execution-count')
@withTwind()
export class UINodeExecutionCount extends LitElement {
  @property({ type: Number })
  value: number = 0

  override render() {
    return html`
      <stencila-ui-node-simple-property
        icon-name="playCircle"
        icon-library="default"
        tooltip-content="Number of times executed"
      >
        ${this.value}
      </stencila-ui-node-simple-property>
    `
  }
}
