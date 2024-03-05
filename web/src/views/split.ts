import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../twind'
import type { DocumentAccess, DocumentId } from '../types'

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
@withTwind()
export class SplitView extends LitElement {
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

  override render() {
    const dynamicStyles = apply([
      'py-11 px-16',
      'max-w-[65ch] lg:max-w-[120ch]',
    ])

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
          <div slot="right" class="${dynamicStyles}">
            <stencila-dynamic-view
            view="dynamic"
            doc=${this.doc}
            access=${this.access}
            theme=${this.theme}
            ></stencila-dynamic-view>
          </div>
          </stencila-dynamic-view>
        </stencila-ui-drag-split>
      </div>
    `
  }
}
