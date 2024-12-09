import { customElement } from 'lit/decorators.js'

import { PreviewClient } from '../clients/preview'

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
  protected previewClient: PreviewClient

  /**
   * Override to pass the render root to the client
   */
  protected override createRenderRoot(): this {
    const renderRoot = super.createRenderRoot()
    this.previewClient = new PreviewClient(renderRoot)
    return renderRoot
  }
}
