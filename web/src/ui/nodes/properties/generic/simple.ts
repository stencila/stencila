import '@shoelace-style/shoelace/dist/components/icon/icon'
import '@shoelace-style/shoelace/dist/components/tooltip/tooltip'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../../twind'

/**
 * A component for displaying a simple, atomic property of a node with an icon
 */
@customElement('stencila-ui-node-simple-property')
@withTwind()
export class UINodeSimpleProperty extends LitElement {
  @property({ attribute: 'icon-name' })
  iconName: string

  @property({ attribute: 'icon-library' })
  iconLibrary: 'stencila' | 'default' = 'stencila'

  @property({ attribute: 'tooltip-content' })
  tooltipContent?: string

  override render() {
    const content = html`<slot></slot>`

    return html`<div class="flex flex-row w-full">
      <div class="pt-0.5">
        <sl-icon
          name=${this.iconName}
          library=${this.iconLibrary}
          class="text-base text-black"
        ></sl-icon>
      </div>
      <div class="grow ml-4">
        <sl-tooltip
          content=${this.tooltipContent}
          placement="top-start"
          .disabled=${(this.tooltipContent?.length ?? 0) === 0}
          ><span>${content}</span></sl-tooltip
        >
      </div>
    </div>`
  }
}
