import { customElement } from 'lit/decorators.js'

import { VSCodeClient } from '../clients/vscode'

import { DocumentView } from './document'

import '../nodes'
import '../shoelace'
import '../ui/document/menu'

/**
 * A view for a VSCode WebView preview panel
 */
@customElement('stencila-vscode-view')
export class VsCodeView extends DocumentView {
  /**
   * Client for handling the messages to and from the VSCode webview API
   */
  protected client: VSCodeClient

  /**
   * Override to pass the render root to the client
   */
  protected override createRenderRoot(): this {
    const renderRoot = super.createRenderRoot()
    this.client = new VSCodeClient(renderRoot)
    return renderRoot
  }
}
