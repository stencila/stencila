import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'
import '../ui/nodes/card'

import { Executable } from './executable'

/**
 * Web component representing a Stencila Schema `IfBlock` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/if-block.md
 */
@customElement('stencila-if-block')
@withTwind()
export class IfBlock extends Executable {
  override render() {
    return html`
      <stencila-ui-block-on-demand
        type="IfBlock"
        depth=${this.depth}
        ancestors=${this.ancestors}
      >
        <span slot="header-right"></span>
        <div slot="body" class="h-full">
          <stencila-ui-node-execution-details
            type="IfBlock"
            mode=${this.executionMode}
            .tags=${this.executionTags}
            status=${this.executionStatus}
            required=${this.executionRequired}
            count=${this.executionCount}
            ended=${this.executionEnded}
            duration=${this.executionDuration}
          >
            <slot name="execution-dependencies"></slot>
            <slot name="execution-dependants"></slot>
          </stencila-ui-node-execution-details>

          <stencila-ui-node-authors type="IfBlock">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>

          <stencila-ui-node-execution-messages
            type="IfBlock"
            message-count=${this.messageCount}
            warning-count=${this.warningCount}
            error-count=${this.errorCount}
          >
            <slot name="execution-messages"></slot>
          </stencila-ui-node-execution-messages>
        </div>
        <div slot="content">
          <slot name="clauses"></slot>
        </div>
      </stencila-ui-block-on-demand>
    `
  }
}
