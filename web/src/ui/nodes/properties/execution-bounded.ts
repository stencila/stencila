import { ExecutionBounds } from '@stencila/types'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../twind'
import { IconName } from '../../icons/icon'

import './generic/simple'

/**
 * A component for displaying the `executionBounded` property of executable nodes
 */
@customElement('stencila-ui-node-execution-bounded')
@withTwind()
export class UINodeExecutionBounded extends LitElement {
  @property()
  value?: ExecutionBounds

  override render() {
    let icon: IconName | null = null
    let tooltip = 'Bounds on last execution'

    switch (this.value) {
      case 'Main': {
        icon = 'arrowNarrowUp'
        tooltip = 'Executed in main kernel'
        break
      }
      case 'Fork': {
        icon = 'arrowRampRight3'
        tooltip = 'Executed in a forked kernel'
        break
      }
      case 'Box': {
        icon = 'box'
        tooltip = 'Executed in a forked and restricted kernel'
        break
      }
    }

    return icon
      ? html`
          <stencila-ui-node-simple-property icon=${icon} tooltip=${tooltip}>
            ${this.value}
          </stencila-ui-node-simple-property>
        `
      : ''
  }
}
