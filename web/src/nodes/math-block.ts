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
  /**
   * In static view, just render the MathML compiled from the `code`
   */
  override renderStaticView() {
    return html`<slot name="mathml"></slot>`
  }

  /**
   * In dynamic view, render a node card with `authors`, `code`
   * and `mathLanguage`, in addition to the compiled MathML.
   */
  override renderDynamicView() {
    return html`
      <stencila-ui-node-card type="MathBlock" view="source">
        <div slot="body">
          <stencila-ui-node-authors type="MathBlock">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>

          <stencila-ui-node-code
            type="MathBlock"
            code=${this.code}
            language=${this.mathLanguage}
            read-only
            collapsed
          >
          </stencila-ui-node-code>

          <div class="p-6">
            <slot name="mathml"></slot>
          </div>
        </div>
      </stencila-ui-node-card>
    `
  }

  /**
   * In source view, render a node card with `authors` and
   * the compiled MathML but do not show the `code` or `mathLanguage`
   * since that is available in the source.
   */
  override renderSourceView() {
    return html`
      <stencila-ui-node-card type="MathBlock" view="source">
        <div slot="body">
          <stencila-ui-node-authors type="MathBlock">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>

          <div class="p-6">
            <slot name="mathml"></slot>
          </div>
        </div>
      </stencila-ui-node-card>
    `
  }
}
