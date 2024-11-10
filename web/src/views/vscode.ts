import { customElement } from 'lit/decorators.js'

import { WebViewClient } from '../clients/webview'

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
  protected webviewClient: WebViewClient

  /**
   * Override to pass the render root to the client
   */
  protected override createRenderRoot(): this {
    const renderRoot = super.createRenderRoot()
    this.webviewClient = new WebViewClient(renderRoot)
    return renderRoot
  }
}
