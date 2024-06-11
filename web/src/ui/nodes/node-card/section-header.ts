import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { ShoelaceIconLibraries } from '../../../shoelace'
import { withTwind } from '../../../twind'

/**
 * UI Node Card Section Header
 *
 * The header element as displayed in node card property elements (e.g. authors).
 */
@customElement('stencila-ui-node-card-section-header')
@withTwind()
export class UISectionHeader extends LitElement {
  @property()
  headerBg: string

  @property({ attribute: 'icon-name' })
  iconName: string

  @property({ attribute: 'icon-library' })
  iconLibrary: ShoelaceIconLibraries = 'stencila'

  @property()
  clickEvent?: () => void

  @property({ attribute: 'wrapper-css' })
  wrapperCss: string | undefined

  override render() {
    return html`<div
      class=${`flex flex-row items-center px-4 py-1.5 font-sans not-italic ${this.headerBg ? `bg-[${this.headerBg}]` : ''}${this.clickEvent ? ' cursor-pointer' : ''} ${this.wrapperCss}`}
      @click=${this.clickEvent}
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

      <slot name="content"></slot>
      <slot name="right-side"></slot>
    </div>`
  }
}
