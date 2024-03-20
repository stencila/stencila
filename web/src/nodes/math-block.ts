import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'

import '../ui/nodes/card'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code'

import { Math } from './math'

@customElement('stencila-math-block')
@withTwind()
export class MathBlock extends Math {
  override renderStaticView() {
    return html`<slot name="mathml"></slot>`
  }

  override renderDynamicView() {
    return html`
      <stencila-ui-node-card type="MathBlock" view="source">
        <div slot="body">
          <stencila-ui-node-authors type="MathBlock">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>

          <stencila-ui-node-code
            type="MathBlock"
            language=${this.mathLanguage}
            read-only
            collapsed
          >
            <slot name="code"></slot>
          </stencila-ui-node-code>

          <div class="p-6">
            <slot name="mathml"></slot>
          </div>
        </div>
      </stencila-ui-node-card>
    `
  }

  override renderSourceView() {
    return html`
      <stencila-ui-node-card type="MathBlock" view="source">
        <div slot="body">
          <stencila-ui-node-authors type="MathBlock">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>

          <stencila-ui-node-code
            type="MathBlock"
            language=${this.mathLanguage}
            read-only
            collapsed
          >
            <slot name="code"></slot>
          </stencila-ui-node-code>

          <slot name="mathml"></slot>
        </div>
      </stencila-ui-node-card>
    `
  }
}
