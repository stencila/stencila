import { LitElement, html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../../../twind'

import './generic/collapsible'

/**
 * A component for displaying the `executionMessages` property of executable nodes
 */
@customElement('stencila-ui-node-execution-messages')
@withTwind()
export class UINodeExecutionMessages extends LitElement {
  // TODO: implement summary icons of the number of messages of each level

  override render() {
    return html`
      <stencila-ui-node-collapsible-property
        .collapsed=${false}
        icon-name="terminal"
        icon-library="default"
      >
        <span slot="title">Messages</span>
        <div class="px-6 py-3 flex flex-col gap-y-3" slot="content">
          <slot></slot>
        </div>
      </stencila-ui-node-collapsible-property>
    `
  }
}
