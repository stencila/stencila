import { ExecutionKind } from '@stencila/types'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../twind'
import { IconName } from '../../icons/icon'

import './generic/simple'

/**
 * A component for displaying the `executionKind` property of executable nodes
 */
@customElement('stencila-ui-node-execution-kind')
@withTwind()
export class UINodeExecutionKind extends LitElement {
  @property()
  value?: ExecutionKind

  override render() {
    let icon: IconName | null = null
    let tooltip = 'Kind of last execution'

    switch (this.value) {
      case 'Main': {
        icon = 'arrowNarrowUp'
        tooltip = 'Executed in the main kernel set'
        break
      }
      case 'Fork': {
        icon = 'arrowRampRight3'
        tooltip = 'Executed in a forked kernel sandbox'
        break
      }
    }

    return icon
      ? html`
          <stencila-ui-node-simple-property
            icon-name=${icon}
            tooltip-content=${tooltip}
          >
            ${this.value}
          </stencila-ui-node-simple-property>
        `
      : ''
  }
}
