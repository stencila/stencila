import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../twind'

import '../ui/nodes/card'
import '../ui/nodes/actions/execution-actions'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/execution-details'

import { CodeExecutable } from './code-executable'

/**
 * Web component representing a Stencila Schema `CodeExpression` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-expression.md
 */
@customElement('stencila-code-expression')
@withTwind()
export class CodeExpression extends CodeExecutable {
  @property()
  label?: string

  /**
   * In static view just render the output, label and caption
   */
  override renderStaticView() {
    return html`<div>
      <stencila-ui-node-output>
        <slot name="output"></slot>
      </stencila-ui-node-output>
      <div>
        <stencila-ui-node-label>${this.label}</stencila-ui-node-label>
        <stencila-ui-node-caption>
          <slot name="caption"></slot>
        </stencila-ui-node-caption>
      </div>
    </div>`
  }

  /**
   * In dynamic view, in addition to what is in static view, render a node card
   * with execution actions and details and code read-only, single-line and collapsed.
   */
  override renderDynamicView() {
    return html`<stencila-ui-node-card type="CodeExpression" view="dynamic">
      <span slot="header-right">
        <stencila-ui-node-execution-actions></stencila-ui-node-execution-actions>
      </span>
      <div slot="body">
        <stencila-ui-node-execution-details
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

        <stencila-ui-node-authors>
          <slot name="authors"></slot>
        </stencila-ui-node-authors>

        <stencila-ui-node-code
          language=${this.programmingLanguage}
          read-only
          single-line
          collapsed
        >
          <slot name="code"></slot>
        </stencila-ui-node-code>

        <stencila-ui-node-execution-messages>
          <slot name="execution-messages"></slot>
        </stencila-ui-node-execution-messages>

        <stencila-ui-node-output>
          <slot name="output"></slot>
        </stencila-ui-node-output>

        <div>
          <stencila-ui-node-label>${this.label}</stencila-ui-node-label>
          <slot name="caption"></slot>
        </div>
      </div>
    </stencila-ui-node-card>`
  }

  /**
   * In source view render everything as in dynamic view in a node card except for
   * code (because it is displayed in the source code).
   */
  override renderSourceView() {
    return html`<stencila-ui-node-card type="CodeExpression" view="source">
      <span slot="header-right">
        <stencila-ui-node-execution-actions>
        </stencila-ui-node-execution-actions>
      </span>
      <div slot="body">
        <stencila-ui-node-execution-details
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

        <stencila-ui-node-authors>
          <slot name="authors"></slot>
        </stencila-ui-node-authors>

        <stencila-ui-node-execution-messages>
          <slot name="execution-messages"></slot>
        </stencila-ui-node-execution-messages>

        <stencila-ui-node-output>
          <slot name="output"></slot>
        </stencila-ui-node-output>
      </div>
    </stencila-ui-node-card>`
  }
}
