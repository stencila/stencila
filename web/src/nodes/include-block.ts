import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import '../ui/nodes/card'

import { Executable } from './executable'

/**
 * Web component representing a Stencila Schema `IncludeBlock` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/include-block.md
 */
@customElement('stencila-include-block')
export class IncludeBlock extends Executable {
  /**
   * path of the file being 'included'
   */
  @property({ type: String })
  source: string

  // TODO: render the source field properly, currently using placeholder

  override render() {
    return html`
      <stencila-ui-block-on-demand
        type="IncludeBlock"
        view="dynamic"
        depth=${this.depth}
        ancestors=${this.ancestors}
      >
        <div slot="body">
          <stencila-ui-node-authors type="IncludeBlock">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>
          <div><span>source: </span><span>${this.source}</span></div>
        </div>
        <div slot="content">
          <slot name="output"></slot>
        </div>
      </stencila-ui-block-on-demand>
    `
  }
}
