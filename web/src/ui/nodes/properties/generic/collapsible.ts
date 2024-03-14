import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import '../../../buttons/chevron'
import { withTwind } from '../../../../twind'

@customElement('stencila-ui-node-collapsible-property')
@withTwind()
export class UINodeCollapsibleProperty extends LitElement {
  @property({ attribute: 'icon-name' })
  iconName: string

  @property({ attribute: 'icon-library' })
  iconLibrary: 'stencila' | 'default' = 'stencila'

  @property({ type: Boolean })
  collapsed: boolean = true

  override render() {
    const contentClasses = apply([
      this.collapsed ? 'max-h-0' : 'max-h-[1000px]',
      'transition-max-h duration-200',
    ])

    return html`
      <div class="overflow-hidden">
        <div class="flex flex-row items-center">
          <sl-icon
            name=${this.iconName}
            library=${this.iconLibrary}
            class="text-base"
          ></sl-icon>
          <div class="grow ml-4">
            <slot name="title"></slot>
          </div>
          <stencila-chevron-button
            .clickEvent=${() => (this.collapsed = !this.collapsed)}
          ></stencila-chevron-button>
        </div>
        <div class=${contentClasses}>
          <slot name="content"></slot>
        </div>
      </div>
    `
  }
}
