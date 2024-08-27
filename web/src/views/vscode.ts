import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { WebViewClient } from '../clients/webview'
import { DocumentPreviewBase } from '../ui/nodes/mixins/preview-base'

import '../nodes'
import '../shoelace'
import '../ui/document/menu'

/**
 * A view for a VSCode WebView preview panel
 *
 * This will use message passing and `morphdom` to update the content.
 */
@customElement('stencila-vscode-view')
export class VsCodeView extends DocumentPreviewBase {
  /**
   * client for handling the messages to and from the vscode webview api
   */
  protected webviewClient: WebViewClient

  protected override createRenderRoot(): this {
    const lightDom = super.createRenderRoot()

    this.webviewClient = new WebViewClient(lightDom)

    return lightDom
  }

  protected override render() {
    return html`
      <slot></slot>
      ${this.renderPreviewMenu()}
    `
  }
}
