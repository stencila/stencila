import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { WebViewClient } from '../clients/webview'

import { DocumentView } from './document'

import '../nodes'
import '../shoelace'
import '../ui/document/menu'

/**
 * A view for a VSCode WebView preview panel
 *
 * This will use message passing and `morphdom` to update the content.
 */
@customElement('stencila-vscode-view')
export class VsCodeView extends DocumentView {
  /**
   * Client for handling the messages to and from the vscode webview API
   */
  protected webviewClient: WebViewClient

  protected override createRenderRoot(): this {
    const renderRoot = super.createRenderRoot()
    this.webviewClient = new WebViewClient(renderRoot)
    return renderRoot
  }  

  protected override render() {
    // The empty root custom element of the correct type needs to be
    // created here for diffs received by the `DomClient` to be applied properly
    const root = html`<stencila-article root></stencila-article>`

    // Menu needs to render after root
    const menu = this.renderDocumentMenu()

    return html`${root}${menu}`
  }
}
