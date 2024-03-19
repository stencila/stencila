import { NodeType } from '@stencila/types'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../twind'

import './generic/collapsible'

@customElement('stencila-ui-node-outputs')
@withTwind()
export class UiNodeOutputs extends LitElement {
  @property()
  type: NodeType

  override render() {
    return html`
      <stencila-ui-node-collapsible-property
        type=${this.type}
        icon-name="output"
        icon-library="stencila"
        .collapsed=${false}
      >
        <span slot="title">Outputs</span>
        <div class="px-6 py-3 flex flex-col gap-y-3" slot="content">
          <slot></slot>
        </div>
      </stencila-ui-node-collapsible-property>
    `
  }
}
