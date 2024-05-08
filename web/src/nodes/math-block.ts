import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'

import '../ui/nodes/node-card/in-flow/block'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code'
import '../ui/nodes/properties/provenance/provenance'

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
      <stencila-ui-block-on-demand type="MathBlock" view="dynamic">
        <div slot="body">
          <stencila-ui-node-authors type="MathBlock">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
          <stencila-ui-node-provenance type="MathBlock">
            <slot name="provenance"></slot>
          </stencila-ui-node-provenance>

          <stencila-ui-node-code
            type="MathBlock"
            code=${this.code}
            language=${this.mathLanguage}
            read-only
          >
          </stencila-ui-node-code>
        </div>
        <div slot="content">
          <div class="px-4 py-3">
            <slot name="mathml"></slot>
          </div>
        </div>
      </stencila-ui-block-on-demand>
    `
  }

  /**
   * In source view, render a node card with `authors` and
   * the compiled MathML but do not show the `code` or `mathLanguage`
   * since that is available in the source.
   */
  override renderSourceView() {
    return html`
      <stencila-ui-block-in-flow type="MathBlock" view="source">
        <div slot="body">
          <stencila-ui-node-authors type="MathBlock">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>

          <div class="p-6">
            <slot name="mathml"></slot>
          </div>
        </div>
      </stencila-ui-block-in-flow>
    `
  }
}
