import '@shoelace-style/shoelace/dist/components/icon/icon'
import '@shoelace-style/shoelace/dist/components/tooltip/tooltip'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../../twind'
import { IconName } from '../../../icons/icon'

/**
 * A component for displaying a simple, atomic property of a node with an icon
 */
@customElement('stencila-ui-node-simple-property')
@withTwind()
export class UINodeSimpleProperty extends LitElement {
  @property({ attribute: 'icon' })
  iconName: IconName

  @property({ attribute: 'tooltip' })
  tooltip?: string

  override render() {
    return html`<sl-tooltip
      content=${this.tooltip}
      placement="top-start"
      .disabled=${(this.tooltip?.length ?? 0) === 0}
    >
      <div class="flex flex-row w-full h-full items-center gap-x-1 shrink-0">
        <stencila-ui-icon
          name=${this.iconName}
          class="text-base"
        ></stencila-ui-icon>
        <div class="grow">
          <slot></slot>
        </div>
      </div>
    </sl-tooltip>`
  }
}
