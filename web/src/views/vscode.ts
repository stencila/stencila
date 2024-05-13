import { LitElement, html } from 'lit'
import { customElement } from 'lit/decorators.js'

import '../nodes'
import '../shoelace'

/**
 * A view for a VSCode WebView preview panel
 *
 * This will use message passing and `morphdom` to update the content.
 */
@customElement('stencila-vscode-view')
export class VsCodeView extends LitElement {
  /**
   * Override so that this component has a Light DOM so that
   * theme styles apply to it.
   */
  protected override createRenderRoot() {
    return this
  }

  override render() {
    return html``
  }
}
