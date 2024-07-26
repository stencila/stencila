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

  @property({ attribute: 'header-title' })
  headerTitle: string

  @property({ type: Boolean })
  collapsed: boolean

  @property({ attribute: 'wrapper-css' })
  wrapperCSS: string | undefined = undefined

  override render() {
    const { colour, borderColour } = nodeUi(this.type)

    const headerStyles = apply([
      'flex flex-row items-center',
      'h-9',
      'px-4 py-1',
      'font-sans not-italic',
      `border-t border-[${borderColour}]`,
      !this.collapsed && `border-b`,
      `bg-[${colour}]`,
      'cursor-pointer',
    ])

    const contentClasses = apply([
      this.collapsed ? 'max-h-0 overflow-hidden' : 'max-h-[150000px]',
      'transition-max-h duration-200',
      `bg-white/50`,
    ])

    return html`
      <div class=${`${this.wrapperCSS ?? ''}`}>
        <div
          class=${headerStyles}
          @click=${() => {
            this.collapsed = !this.collapsed
          }}
        >
          <sl-icon
            name=${this.iconName}
            library=${this.iconLibrary}
            class="text-base"
          ></sl-icon>

          <span class="grow ml-2 select-none text-xs">
            <span>${this.headerTitle}</span>
          </span>

          <slot name="header-content"></slot>

          <stencila-chevron-button
            default-pos=${this.collapsed ? 'left' : 'down'}
            custom-class="flex items-center"
          ></stencila-chevron-button>
        </div>

        <div class=${contentClasses}>
          <slot name="content"></slot>
        </div>
      </div>
    `
  }
}
