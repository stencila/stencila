import { NodeType } from '@stencila/types'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../twind'
import { nodeUi } from '../icons-and-colours'

@customElement('stencila-ui-node-outputs')
@withTwind()
export class NodeOutputs extends LitElement {
  @property({ type: String })
  type: NodeType

  override render() {
    const { borderColour } = nodeUi(this.type)

    return html`
      <stencila-ui-node-collapsible-property
        .collapsed=${false}
        icon-name="output"
        icon-library="stencila"
        header-bg=${borderColour}
      >
        <span slot="title">Output</span>
        <div class="px-6 py-3 flex flex-col gap-y-3" slot="content">
          <slot></slot>
        </div>
      </stencila-ui-node-collapsible-property>
    `
  }
}
