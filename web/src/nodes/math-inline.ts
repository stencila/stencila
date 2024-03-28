import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'

import '../ui/nodes/on-demand/in-line'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code'

import { Math } from './math'

@customElement('stencila-math-inline')
@withTwind()
export class MathInline extends Math {
  /**
   * In static view, just render the MathML compiled from the `code`
   */
  override renderStaticView() {
    return html`<slot name="mathml"></slot>`
  }

  /**
   * In dynamic view, render a node card on demand with `authors`, `code`
   * and `mathLanguage`. The compiled MathML is always shown.
   */
  override renderDynamicView() {
    return html`
      <stencila-ui-inline-on-demand type="MathInline" view="source">
        <div slot="body">
          <stencila-ui-node-authors type="MathInline">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>

          <stencila-ui-node-code
            type="MathInline"
            code=${this.code}
            language=${this.mathLanguage}
            read-only
            collapsed
          >
          </stencila-ui-node-code>
        </div>
        <span slot="content">
          <slot name="mathml"></slot>
        </span>
      </stencila-ui-inline-on-demand>
    `
  }

  /**
   * In source view, render a node card with `authors` and the compiled MathML.
   * No need to show the `code` since that is available in the source.
   */
  override renderSourceView() {
    return html`
      <stencila-ui-node-card type="MathInline" view="source">
        <div slot="body">
          <stencila-ui-node-authors type="MathInline">
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
