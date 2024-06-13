import { NodeType } from '@stencila/types'
import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import '../../../buttons/chevron'
import '../../node-card/section-header'
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
  collapsed: boolean = true

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
        <stencila-ui-node-card-section-header
          .clickEvent=${() => {
            this.collapsed = !this.collapsed
          }}
          icon-name=${this.iconName}
          icon-library=${this.iconLibrary}
          headerBg=${headerBg}
        >
          <div slot="title">
            <slot name="title"></slot>
          </div>
          <div slot="content">
            <slot name="header-content"></slot>
          </div>
          <stencila-chevron-button
            default-pos=${this.collapsed ? 'left' : 'down'}
            slot="right-side"
            custom-class="flex items-center"
          ></stencila-chevron-button>
        </stencila-ui-node-card-section-header>
        <div class=${contentClasses}>
          <slot name="content"></slot>
        </div>
      </div>
    `
  }
}
