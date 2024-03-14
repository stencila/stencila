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
        icon-name="terminal"
        icon-library="default"
        class="my-1"
      >
        <span slot="title">Messages</span>
        <div slot="content">
          <slot></slot>
        </div>
      </stencila-ui-node-collapsible-property>
    `
  }
}
