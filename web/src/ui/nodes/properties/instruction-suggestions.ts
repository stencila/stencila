import { NodeType } from '@stencila/types'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../twind'

import './generic/collapsible'

/**
 * A component for displaying the `suggestions` property of `Instruction` nodes
 */
@customElement('stencila-ui-node-instruction-suggestions')
@withTwind()
export class UINodeInstructionSuggestions extends LitElement {
  @property()
  type: NodeType

  override render() {
    return html`
      <stencila-ui-node-collapsible-property
        type=${this.type}
        icon-name="lightbulb"
        icon-library="default"
      >
        <div slot="title">
          <span>Suggestions</span>
        </div>
        <div slot="content" class="p-3 font-sans text-sm">
          <slot></slot>
        </div>
      </stencila-ui-node-collapsible-property>
    `
  }
}
