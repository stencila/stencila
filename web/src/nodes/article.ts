import { LitElement, html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'

import '../ui/nodes/properties/authors-provenance'

@customElement('stencila-article')
@withTwind()
export class StencilaArticle extends LitElement {
  override render() {
    return html`
      <aside class="min-w-80 max-w-prose mx-auto">
        <stencila-ui-authors-provenance>
          <div class="flex flex-col gap-y-4">
            <div slot="authors">
              <slot name="authors"></slot>
            </div>
            <div slot="provenance">
              <slot name="provenance"></slot>
            </div>
          </div>
        </stencila-ui-authors-provenance>
      </aside>

      <!-- TODO: <stencila-ui-article-headings> component -->
      <div style="display:none">
        <slot name="headings"></slot>
      </div>

      <slot name="content"></slot>
    `
  }
}
