import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'

import '../ui/nodes/cards/inline-on-demand'
import '../ui/nodes/cards/block-in-flow'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/provenance'

import { Math } from './math'

@customElement('stencila-math-inline')
@withTwind()
export class MathInline extends Math {
  /**
   * render a node card on demand with `authors`, `code`
   * and `mathLanguage`. The compiled MathML is always shown.
   */
  override render() {
    return html`
      <stencila-ui-inline-on-demand type="MathInline">
        <div slot="body">
          <stencila-ui-node-authors type="MathInline">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>

          <stencila-ui-node-code
            type="MathInline"
            code=${this.code}
            node-id=${this.id}
            .code-authorship=${this.codeAuthorship}
            language=${this.mathLanguage ?? 'tex'}
            read-only
          >
          </stencila-ui-node-code>
        </div>
        <span slot="content">
          <slot name="mathml"></slot>
          <slot name="images"></slot>
        </span>
      </stencila-ui-inline-on-demand>
    `
  }
}
