import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import '../ui/nodes/node-card/on-demand/in-line'
import '../ui/nodes/commands/execution-commands'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/execution-details'
import '../ui/nodes/properties/execution-messages'
import '../ui/nodes/properties/outputs'

import { CodeExecutable } from './code-executable'

/**
 * Web component representing a Stencila Schema `CodeExpression` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-expression.md
 */
@customElement('stencila-code-expression')
@withTwind()
export class CodeExpression extends CodeExecutable {
  /**
   * In dynamic view, in addition to what is in static view, render a node card
   * on demand with execution actions and details and code read-only.
   */
  override render() {
    return html`<stencila-ui-inline-on-demand
      type="CodeExpression"
      view="dynamic"
      programming-language=${this.programmingLanguage}
    >
      <span slot="header-right">
        <stencila-ui-node-execution-commands
          node-id=${this.id}
          type="CodeExpression"
        >
        </stencila-ui-node-execution-commands>
      </span>
      <div slot="body">
        <stencila-ui-node-execution-details
          type="CodeExpression"
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

        <stencila-ui-node-authors type="CodeExpression">
          <stencila-ui-node-provenance slot="provenance">
            <slot name="provenance"></slot>
          </stencila-ui-node-provenance>
          <slot name="authors"></slot>
        </stencila-ui-node-authors>

        <stencila-ui-node-code
          type="CodeExpression"
          code=${this.code}
          code-authorship=${this.codeAuthorship}
          language=${this.programmingLanguage}
          read-only
        >
          <slot name="execution-messages"></slot>
        </stencila-ui-node-code>

        <stencila-ui-node-execution-messages
          type="CodeExpression"
          message-count=${this.messageCount}
          warning-count=${this.warningCount}
          error-count=${this.errorCount}
        >
          <slot name="execution-messages"></slot>
        </stencila-ui-node-execution-messages>
      </div>
      <span slot="content">
        ${this.executionCount > 0 ? html`<slot name="output"></slot>` : ''}
      </span>
    </stencila-ui-inline-on-demand>`
  }
}
