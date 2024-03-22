import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import '../ui/nodes/card'
import '../ui/nodes/commands/execution-commands'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code'
import '../ui/nodes/properties/execution-details'
import '../ui/nodes/properties/execution-messages'
import '../ui/nodes/properties/output'

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
   * In static view just render the output without a wrapping
   * `<stencila-ui-node-output>`
   */
  override renderStaticView() {
    return html`<slot name="output"></slot>`
  }

  /**
   * In dynamic view, in addition to what is in static view, render a node card
   * on demand with execution actions and details and code read-only.
   */
  override renderDynamicView() {
    return html`<stencila-ui-node-card
      type="CodeExpression"
      view="dynamic"
      display="on-demand"
    >
      <span slot="header-right">
        <stencila-ui-node-execution-commands node-id=${this.id}>
        </stencila-ui-node-execution-commands>
      </span>
      <div slot="body">
        <stencila-ui-node-execution-details
          type="CodeExpression"
          auto-exec=${this.autoExec}
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
          <slot name="authors"></slot>
        </stencila-ui-node-authors>

        <stencila-ui-node-code
          type="CodeExpression"
          code=${this.code}
          language=${this.programmingLanguage}
          read-only
          collapsed
        >
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
        <stencila-ui-node-output type="CodeExpression">
          <slot name="output"></slot>
        </stencila-ui-node-output>
      </span>
    </stencila-ui-node-card>`
  }

  /**
   * In source view render everything as in dynamic view in a node card except for
   * code (because it is displayed in the source code).
   */
  override renderSourceView() {
    return html`<stencila-ui-node-card type="CodeExpression" view="source">
      <span slot="header-right">
        <stencila-ui-node-execution-commands node-id=${this.id}>
        </stencila-ui-node-execution-commands>
      </span>
      <div slot="body">
        <stencila-ui-node-execution-details
          type="CodeExpression"
          auto-exec=${this.autoExec}
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
          <slot name="authors"></slot>
        </stencila-ui-node-authors>

        <stencila-ui-node-execution-messages
          type="CodeExpression"
          message-count=${this.messageCount}
          warning-count=${this.warningCount}
          error-count=${this.errorCount}
        >
          <slot name="execution-messages"></slot>
        </stencila-ui-node-execution-messages>

        <stencila-ui-node-output>
          <slot name="output"></slot>
        </stencila-ui-node-output>
      </div>
    </stencila-ui-node-card>`
  }
}
