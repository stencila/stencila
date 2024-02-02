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
        ?active=${this.context.filesOpen}
      ></stencila-ui-icon-button>
      <div class="flex flex-col space-y-9 text-xl">
        <stencila-ui-icon-button
          icon="live-view"
          .clickEvent=${() => {
            this.createEvent('stencila-view-change', {
              view: 'live',
            })
          }}
          ?active=${this.context.view === 'live'}
          type="selected"
        ></stencila-ui-icon-button>
        <stencila-ui-icon-button
          icon="code-view"
          .clickEvent=${() => {
            this.createEvent('stencila-view-change', {
              view: 'source',
            })
          }}
          ?active=${this.context.view === 'source'}
        ></stencila-ui-icon-button>
        <stencila-ui-icon-button
          icon="split-view"
          .clickEvent=${() => {
            this.createEvent('stencila-view-change', {
              view: 'split',
            })
          }}
          ?active=${this.context.view === 'split'}
        ></stencila-ui-icon-button>
        <stencila-ui-icon-button icon="settings"></stencila-ui-icon-button>
      </div>
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
