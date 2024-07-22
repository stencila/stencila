import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import '../ui/nodes/card'
import '../ui/nodes/commands/execution-commands'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/execution-details'

import { CodeExecutable } from './code-executable'

/**
 * Web component representing a Stencila Schema `For` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/for-block.md
 */
@customElement('stencila-for-block')
@withTwind()
export class ForBlock extends CodeExecutable {
  override render() {
    return html`
      <stencila-ui-block-on-demand
        type="ForBlock"
        view="dynamic"
        depth=${this.depth}
        ancestors=${this.ancestors}
      >
        <span slot="header-right">
          <stencila-ui-node-execution-commands
            node-id=${this.id}
            type="ForBlock"
          >
          </stencila-ui-node-execution-commands>
        </span>
        <div slot="body" class="h-full">
          <stencila-ui-node-execution-details
            type="ForBlock"
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

          <stencila-ui-node-authors type="ForBlock">
            <stencila-ui-node-provenance slot="provenance">
              <slot name="provenance"></slot>
            </stencila-ui-node-provenance>
            <slot name="authors"></slot>
          </stencila-ui-node-authors>

          <stencila-ui-node-code
            type="ForBlock"
            code=${this.code}
            language=${this.programmingLanguage}
            read-only
          >
            <slot name="execution-messages" slot="execution-messages"></slot>
          </stencila-ui-node-code>
        </div>
        <div slot="content">
          <slot name="iterations"></slot>
        </div>
      </stencila-ui-block-on-demand>
    `
  }
}
