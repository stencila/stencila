import { consume } from '@lit/context'
import { LitElement, html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { SidebarContext, sidebarContext } from '../../contexts/sidebar-context'
import { emitSidebarEvent } from '../../events/sidebar'
import { withTwind } from '../../twind'
import { MainContextEvent } from '../../types'

import '../buttons/icon'

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
        tooltip="${this.context.directoryOpen ? 'Close' : 'Open'} file explorer"
        tooltip-placement=${'right'}
        .clickEvent=${() => {
          this.emitEvent('stencila-directory-toggle', {
            directoryOpen: !this.context.directoryOpen,
          })
        }}
        ?active=${this.context.directoryOpen}
      ></stencila-ui-icon-button>
      <div class="flex flex-col space-y-9 text-xl">
        <stencila-ui-icon-button
          icon="visual-view"
          tooltip="Visual editor"
          tooltip-placement=${'right'}
          .clickEvent=${() => {
            this.emitEvent('stencila-view-change', {
              currentView: 'visual',
            })
          }}
          ?active=${this.context.currentView === 'visual'}
          type="selected"
        ></stencila-ui-icon-button>
        <stencila-ui-icon-button
          icon="code-view"
          tooltip="Source editor"
          tooltip-placement=${'right'}
          .clickEvent=${() => {
            this.emitEvent('stencila-view-change', {
              currentView: 'source',
            })
          }}
          ?active=${this.context.currentView === 'source'}
        ></stencila-ui-icon-button>
        <stencila-ui-icon-button
          icon="split-view"
          tooltip="Split view"
          tooltip-placement=${'right'}
          .clickEvent=${() => {
            this.emitEvent('stencila-view-change', {
              currentView: 'split',
            })
          }}
          ?active=${this.context.currentView === 'split'}
        ></stencila-ui-icon-button>
        <stencila-ui-icon-button
          icon="settings"
          tooltip="Settings"
          tooltip-placement=${'right'}
          .clickEvent=${() => {
            this.emitEvent('stencila-config-toggle', {
              configOpen: !this.context.configOpen,
            })
          }}
          ?active=${this.context.configOpen}
        ></stencila-ui-icon-button>
      </div>
    </div> `
  }

  /**
   * Emit a `MainContextEvent`
   *
   * @param name The name of the event
   * @param detail The event details
   */
  private emitEvent<T extends keyof SidebarContext>(
    name: MainContextEvent,
    detail?: Pick<SidebarContext, T>
  ): void {
    const event = emitSidebarEvent(name, detail)
    this.dispatchEvent(event)
  }
}
