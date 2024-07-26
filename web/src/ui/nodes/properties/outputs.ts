import { NodeType } from '@stencila/types'
import { LitElement, html } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { withTwind } from '../../../twind'

import './generic/collapsible'

@customElement('stencila-ui-node-outputs')
@withTwind()
export class UiNodeOutputs extends LitElement {
  @property()
  type: NodeType

  @state()
  private hasItems = false

  override firstUpdated(changedProperties: Map<string, string | boolean>) {
    super.firstUpdated(changedProperties)

    const slot: HTMLSlotElement = this.shadowRoot.querySelector('slot')
    if (slot) {
      this.hasItems = slot.assignedElements({ flatten: true }).length !== 0
    }
  }

  override render() {
    return html`
      <stencila-ui-node-collapsible-property
        type=${this.type}
        icon-name="output"
        icon-library="stencila"
        header-title="Outputs"
        wrapper-css=${this.hasItems ? '' : 'hidden'}
        .collapsed=${false}
      >
        <div slot="content" class="px-4 py-3 flex flex-col gap-y-1">
          <slot></slot>
        </div>
      </stencila-ui-node-collapsible-property>
    `
  }
}
