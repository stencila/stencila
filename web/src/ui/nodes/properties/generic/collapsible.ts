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

  @property({ attribute: 'header-bg' })
  headerBg: string | undefined = undefined

  @property({ attribute: 'wrapper-css' })
  wrapperCSS: string | undefined = undefined

  override render() {
    const contentClasses = apply([
      this.collapsed ? 'max-h-0' : 'max-h-[1000px]',
      'transition-max-h duration-200',
    ])

    return html`
      <div
        class=${`overflow-hidden ${this.wrapperCSS ?? ''}`}
        @click=${() => (this.collapsed = !this.collapsed)}
      >
        <div
          class=${`flex flex-row items-center px-6 py-3 cursor-pointer ${this.headerBg ? `bg-[${this.headerBg}]` : ''}`}
        >
          ${this.iconName &&
          html`<sl-icon
            name=${this.iconName}
            library=${this.iconLibrary}
            class="text-base"
          ></sl-icon>`}

          <div class=${`grow select-none ${this.iconName && 'ml-4'}`}>
            <slot name="title"></slot>
          </div>
          <stencila-chevron-button
            .position=${this.collapsed ? 'left' : 'down'}
          ></stencila-chevron-button>
        </div>
        <div class=${contentClasses}>
          <slot name="content"></slot>
        </div>
      </div>
    `
  }
}
