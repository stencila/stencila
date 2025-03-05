import { html, LitElement } from 'lit'
import { customElement } from 'lit/decorators.js'

import { VSCodeClient } from '../clients/vscode'
import { withTwind } from '../twind'

import { DocumentView } from './document'

import '../nodes'
import '../shoelace'
import '@shoelace-style/shoelace/dist/components/spinner/spinner'
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

/**
 * Loader element to display whilst webview window is loading
 */
@customElement('stencila-vscode-loader')
@withTwind()
export class VsCodeLoader extends LitElement {
  protected override render(): unknown {
    return html`
      <div
        class="fixed inset-0 w-screen h-screen flex items-center justify-center bg-white z-10"
      >
        <sl-spinner style="font-size: 5rem; --track-width: 5px;"></sl-spinner>
      </div>
    `
  }
}
