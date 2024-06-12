import { apply } from '@twind/core'
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
    const styles = apply([
      'flex flex-row items-center',
      'h-9',
      'px-4 py-1',
      'font-sans not-italic',
      this.headerBg && `bg-[${this.headerBg}]`,
      this.clickEvent && 'cursor-pointer',
      'border-t border-black/20',
      this.wrapperCss ?? '',
    ])

    return html`
      <div class=${styles} @click=${this.clickEvent}>
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
      </div>
    `
  }
}
