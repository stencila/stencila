import { consume } from '@lit/context'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { SidebarContext, sidebarContext } from '../../contexts/sidebar-context'
import { withTwind } from '../../twind'
import { DocumentId } from '../../types'

/**
 * UI File tree layout
 *
 * Wraps our directory tree in a UI element and interacts with the context
 */
@customElement('stencila-ui-file-tree-layout')
@withTwind()
export class UIFileTreeLayout extends LitElement {
  @property()
  doc?: DocumentId

  @consume({ context: sidebarContext, subscribe: true })
  context: SidebarContext

  override render() {
    return html` <div
      class="${this.context?.filesOpen
        ? 'block'
        : 'hidden'} mr-4 w-80 overflow-x-hidden h-full"
    >
      <stencila-directory-view></stencila-directory-view>
    </div>`
  }
}
