import { consume } from '@lit/context'
import { LitElement, html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { SidebarContext, sidebarContext } from '../../contexts/sidebar-context'
import { withTwind } from '../../twind'

/**
 * UI directory view container
 *
 * Wraps `<stencila-directory-view>` in a UI element and interacts with the
 * sidebar context to show or hide it.
 */
@customElement('stencila-ui-directory-container')
@withTwind()
export class UIDirectory extends LitElement {
  @consume({ context: sidebarContext, subscribe: true })
  context: SidebarContext

  override render() {
    return html` <div
      class="${this.context?.filesOpen
        ? 'block'
        : 'hidden'} mr-4 w-80 overflow-x-hidden h-full"
    >
      <stencila-directory-view
        class="flex flex-col h-screen justify-end"
      ></stencila-directory-view>
    </div>`
  }
}
