import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'
import { getTitleIcon } from '../ui/nodes/properties/programming-language'

import '../ui/nodes/commands/execution-commands'
import '../ui/nodes/cards/inline-on-demand'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/execution-details'
import '../ui/nodes/properties/execution-messages'
import '../ui/nodes/properties/provenance'

import { CodeExecutable } from './code-executable'

/**
 * Web component representing a Stencila Schema `CodeExpression` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-expression.md
 */
@customElement('stencila-code-expression')
@withTwind()
export class CodeExpression extends CodeExecutable {
  override render() {
    const { icon, title } = getTitleIcon(this.programmingLanguage) ?? {
      title: 'Code',
      icon: 'code',
    }

    return html`<stencila-ui-inline-on-demand
      type="CodeExpression"
      programming-language=${this.programmingLanguage}
      header-icon=${icon}
      header-title="${title} Expression"
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
          .code-authorship=${this.codeAuthorship}
          language=${this.programmingLanguage}
          execution-required=${this.executionRequired}
          read-only
        >
          <slot name="execution-messages" slot="messages"></slot>
        </stencila-ui-node-code>
      </div>
      <span slot="content">
        ${this.executionCount > 0 ? html`<slot name="output"></slot>` : ''}
      </span>
    </stencila-ui-inline-on-demand>`
  }
}
