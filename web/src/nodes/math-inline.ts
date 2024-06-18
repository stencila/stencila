import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { withTwind } from '../twind'

import '../ui/nodes/node-card/on-demand/in-line'
import '../ui/nodes/node-card/in-flow/block'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/provenance/provenance'

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
      <stencila-ui-inline-on-demand type="MathInline" view="dynamic">
        <div slot="body">
          <stencila-ui-node-authors type="MathInline">
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
          <stencila-ui-node-provenance type="MathInline">
            <slot name="provenance"></slot>
          </stencila-ui-node-provenance>

          <stencila-ui-node-code
            type="MathInline"
            code=${this.code}
            code-authorship=${this.codeAuthorship}
            language=${this.mathLanguage ?? 'tex'}
            read-only
          >
          </stencila-ui-node-code>
        </div>
        <span slot="content">
          <slot name="mathml"></slot>
        </span>
      </stencila-ui-inline-on-demand>
    `
  }
}
