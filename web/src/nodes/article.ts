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
            <slot name="authors"></slot>
            <slot name="provenance"></slot>
          </div>
        </stencila-ui-authors-provenance>
      </aside>
      <slot name="content"></slot>
    `
  }
}
