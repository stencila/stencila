import { provide } from '@lit/context'
import { LitElement, html } from 'lit'
import { customElement, state } from 'lit/decorators.js'

import {
  DocPreviewContext,
  documentPreviewContext,
} from '../contexts/preview-context'
import '../nodes'
import '../shoelace'
import '../ui/preview-menu'

/**
 * A view for a VSCode WebView preview panel
 *
 * This will use message passing and `morphdom` to update the content.
 */
@customElement('stencila-vscode-view')
export class VsCodeView extends LitElement {
  @provide({ context: documentPreviewContext })
  @state()
  protected context: DocPreviewContext = {
    showAllToggleChips: false,
  }

  @state()
  private showMenu: boolean = false

  /**
   * Override so that this component has a Light DOM so that
   * theme styles apply to it.
   */
  protected override createRenderRoot() {
    this.addEventListener('mouseenter', () => {
      this.showMenu = true
    })

    this.addEventListener('mouseleave', () => {
      this.showMenu = false
    })

    this.addEventListener('toggle-card-chips', () => {
      this.context = {
        ...this.context,
        showAllToggleChips: !this.context.showAllToggleChips,
      }
    })
    return this
  }

  override render() {
    return html`
      <slot></slot>
      <preview-menu
        ?visible=${this.showMenu}
        ?show-toggle-chips=${this.context.showAllToggleChips}
      ></preview-menu>
    `
  }
}
