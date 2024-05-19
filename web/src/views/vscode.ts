import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import '../nodes'
import '../shoelace'
import '../ui/preview-menu'
import { DocumentPreviewBase } from '../ui/nodes/mixins/preview-base'

/**
 * A view for a VSCode WebView preview panel
 *
 * This will use message passing and `morphdom` to update the content.
 */
@customElement('stencila-vscode-view')
export class VsCodeView extends DocumentPreviewBase {
  protected override render() {
    return html`
      <slot></slot>
      ${this.renderPreviewMenu()}
    `
  }
}
