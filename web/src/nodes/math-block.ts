import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'

import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/provenance'

import { Math } from './math'

@customElement('stencila-math-block')
@withTwind()
export class MathBlock extends Math {
  /**
   * render a node card with `authors`, `code`
   * and `mathLanguage`, in addition to the compiled MathML.
   */
  override render() {
    if (this.ancestors.includes('StyledBlock')) {
      return html`
        <div class="px-4 py-3">
          <slot name="mathml"></slot>
        </div>
      `
    }

    return html`
      <stencila-ui-block-on-demand
        type="MathBlock"
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
            .code-authorship=${this.codeAuthorship}
            language=${this.mathLanguage ?? 'tex'}
            read-only
          >
            <slot name="compilation-messages" slot="messages"></slot>
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
