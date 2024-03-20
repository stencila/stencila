import { NodeType } from '@stencila/types'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../twind'

import './generic/collapsible'

/**
 * A component for displaying the `suggestion` property of `Instruction` nodes
 */
@customElement('stencila-ui-node-suggestion')
@withTwind()
export class UINodeSuggestion extends LitElement {
  @property()
  type: NodeType

  override render() {
    return html`
      <stencila-ui-node-collapsible-property
        .collapsed=${false}
        type=${this.type}
        icon-name="plus-square"
        icon-library="default"
      >
        <div slot="title">
          <span>Suggestion</span>
        </div>
        <div class="p-3" slot="content">
          <slot></slot>
        </div>
      </stencila-ui-node-collapsible-property>
    `
  }
}
