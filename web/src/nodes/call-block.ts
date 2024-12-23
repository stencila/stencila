import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import '../ui/nodes/commands/execution-commands'
import '../ui/nodes/cards/block-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/execution-details'
import '../ui/nodes/properties/execution-messages'
import '../ui/nodes/properties/provenance'

import { IncludeBlock } from './include-block'

/**
 * Web component representing a Stencila Schema `CallBlock` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/call-block.md
 */
@customElement('stencila-call-block')
@withTwind()
export class CallBlock extends IncludeBlock {
  override render() {
    if (
      this.ancestors.includes('StyledBlock') ||
      this.isUserChatMessageNode()
    ) {
      return this.renderContent()
    }

    return html`
      <stencila-ui-block-on-demand
        type="CallBlock"
        depth=${this.depth}
        ancestors=${this.ancestors}
      >
        <span slot="header-right">
          <stencila-ui-node-execution-commands
            type="CallBlock"
            node-id=${this.id}
          >
          </stencila-ui-node-execution-commands>
        </span>

        <div slot="body">
          <stencila-ui-node-execution-details
            type="CallBlock"
            node-id=${this.id}
            mode=${this.executionMode}
            bounds=${this.executionBounds}
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

          <stencila-ui-node-authors type="CallBlock">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>

          ${this.renderSource('CallBlock')}

          <slot name="arguments"></slot>
          <stencila-ui-node-execution-messages type=${'CallBlock'}>
            <slot name="execution-messages"></slot>
          </stencila-ui-node-execution-messages>
        </div>

        <div slot="content">${this.renderContent()}</div>
      </stencila-ui-block-on-demand>
    `
  }
}
