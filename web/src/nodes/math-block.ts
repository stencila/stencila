import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'

import '../ui/nodes/node-card/in-flow/block'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/provenance/provenance'

import { Math } from './math'

@customElement('stencila-math-block')
@withTwind()
export class MathBlock extends Math {
  /**
   * render a node card with `authors`, `code`
   * and `mathLanguage`, in addition to the compiled MathML.
   */
  override render() {
    return html`
      <stencila-ui-block-on-demand
        type="MathBlock"
        view="dynamic"
        depth=${this.depth}
        ancestors=${this.ancestors}
      >
        <div slot="body">
          <stencila-ui-node-authors type="MathBlock">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>

          <stencila-ui-node-code
            type="MathBlock"
            code=${this.code}
            code-authorship=${this.codeAuthorship}
            language=${this.mathLanguage ?? 'tex'}
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
}
