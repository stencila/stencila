import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import type { DocumentAccess, DocumentId } from '../types'
import { TWLitElement } from '../ui/twind'

import './source'
import './dynamic'

import '../ui/split-drag'

/**
 * Split panes view for a document
 *
 * Currently the type of view in each pane is fixed
 * but it is possible that in the future the user could choose
 * between alternative views in each pane.
 */
@customElement('stencila-split-view')
export class SplitView extends TWLitElement {
  /**
   * The id of the document
   */
  @property()
  doc: DocumentId

  /**
   * The access level of the view
   *
   * Passed through to child views.
   */
  @property()
  access: DocumentAccess = 'code'

  /**
   * The format of the source code editor
   */
  @property()
  format: string

  /**
   * The theme to apply to any themed child views in this view
   */
  @property()
  theme: string = 'default'

  protected render() {
    return html`
      <div class="max-h-screen">
        <stencila-ui-drag-split>
          <stencila-source-view
            view="source"
            doc=${this.doc}
            access="write"
            format=${this.format}
            displayMode="split"
            slot="left"
          >
          </stencila-source-view>
          <stencila-dynamic-view
            view="dynamic"
            doc=${this.doc}
            access=${this.access}
            theme=${this.theme}
            slot="right"
          >
          </stencila-dynamic-view>
        </stencila-ui-drag-split>
      </div>
    `
  }
}
