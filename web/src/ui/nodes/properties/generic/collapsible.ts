import { NodeType } from '@stencila/types'
import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import '../../../buttons/chevron'
import { ShoelaceIconLibraries } from '../../../../shoelace'
import { withTwind } from '../../../../twind'
import { nodeUi } from '../../icons-and-colours'

@customElement('stencila-ui-node-collapsible-property')
@withTwind()
export class UINodeCollapsibleProperty extends LitElement {
  @property()
  type: NodeType

  @property({ attribute: 'icon-name' })
  iconName: string

  @property({ attribute: 'icon-library' })
  iconLibrary: ShoelaceIconLibraries = 'stencila'

  @property({ type: Boolean })
  collapsed: boolean = false

  @property({ attribute: 'wrapper-css' })
  wrapperCSS: string | undefined = undefined

  override render() {
    const { borderColour: headerBg } = nodeUi(this.type)

    const contentClasses = apply([
      this.collapsed ? 'max-h-0 overflow-hidden' : 'max-h-[150000px]',
      'transition-max-h duration-200',
    ])

    return html`
      <div class=${`${this.wrapperCSS ?? ''}`}>
        <div
          class=${`flex flex-row items-center px-4 py-1.5 cursor-pointer font-sans not-italic ${headerBg ? `bg-[${headerBg}]` : ''}`}
          @click=${() => (this.collapsed = !this.collapsed)}
        >
          ${this.iconName &&
          html`<sl-icon
            name=${this.iconName}
            library=${this.iconLibrary}
            class="text-base"
          ></sl-icon>`}

          <div class=${`grow select-none text-sm ${this.iconName && 'ml-4'}`}>
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
