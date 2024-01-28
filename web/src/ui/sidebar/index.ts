// import SlMenuItem from '@shoelace-style/shoelace/dist/components/menu-item/menu-item.component.js'
import { apply, css } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../../twind'

/**
 * UI sidebar
 *
 * The sidebar displayed in the main UI
 */
@customElement('stencila-ui-sidebar')
@withTwind()
export class UISidebar extends LitElement {
  override render() {
    const classes = apply(['group'])
    const styles = css`
      &::part(base) {
        border: none;
        line-height: 0;
        min-height: 0;
        background: none;
      }

      &::part(label) {
        padding: 0;
      }
    `

    return html`<div
      class="w-16 flex flex-col items-center justify-between mt-20 h-full max-h-[calc(100vh-5rem)] pb-5"
    >
      <sl-button class="${classes} ${styles}"
        >${this.renderIcon('sidebar')}</sl-button
      >
      <sl-button class="${classes} ${styles}"
        >${this.renderIcon('settings')}</sl-button
      >
    </div> `
  }

  renderIcon(icon: string) {
    return html`<sl-icon
      library="stencila"
      name="${icon}"
      class="text-xl transition-colors stroke-black group-hover:stroke-brand-blue"
    ></sl-icon>`
  }
}
