import { consume } from '@lit/context'
import { LitElement, html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { SidebarContext, sidebarContext } from '../../contexts/sidebar-context'
import { withTwind } from '../../twind'
import { MainContextEvent } from '../../types'

/**
 * UI sidebar
 *
 * The sidebar displayed in the main UI
 */
@customElement('stencila-ui-sidebar')
@withTwind()
export class UISidebar extends LitElement {
  @consume({ context: sidebarContext, subscribe: true })
  context: SidebarContext

  override render() {
    return html`<div
      class="w-16 flex flex-col items-center justify-between mt-20 h-full max-h-[calc(100vh-5rem)] pb-5"
    >
      <stencila-ui-icon-button
        icon="sidebar"
        .clickEvent=${() => {
          this.createEvent('stencila-file-toggle', {
            filesOpen: !this.context.filesOpen,
          })
        }}
      ></stencila-ui-icon-button>
      <stencila-ui-icon-button icon="settings"></stencila-ui-icon-button>
    </div> `
  }

  private createEvent<T extends keyof SidebarContext>(
    name: MainContextEvent,
    detail?: Pick<SidebarContext, T>
  ): void {
    const event = new CustomEvent(name, {
      bubbles: true,
      composed: true,
      detail,
    })

    this.dispatchEvent(event)
  }
}
