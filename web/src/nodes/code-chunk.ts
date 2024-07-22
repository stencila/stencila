import { LabelType } from '@stencila/types'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../twind'

import '../ui/nodes/node-card/in-flow/block'
import '../ui/nodes/commands/execution-commands'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/execution-details'
import '../ui/nodes/properties/execution-messages'
import '../ui/nodes/properties/label-and-caption'
import '../ui/nodes/properties/outputs'
import '../ui/nodes/properties/provenance/provenance'

import { CodeExecutable } from './code-executable'

/**
 * Web component representing a Stencila Schema `CodeChunk` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-chunk.md
 */
@customElement('stencila-code-chunk')
@withTwind()
export class CodeChunk extends CodeExecutable {
  @property({ attribute: 'label-type' })
  labelType?: LabelType

  @property()
  label?: string

  @property({ attribute: 'is-invisible', type: Boolean })
  isInvisible?: boolean = false

  /**
   * In dynamic view, in addition to what is in static view, render a node card
   * with execution actions and details and code read-only and collapsed.
   */
  override render() {
    return html`<stencila-ui-block-on-demand
      type="CodeChunk"
      view="dynamic"
      programming-language="${this.programmingLanguage}"
      depth=${this.depth}
      ancestors=${this.ancestors}
      node-id=${this.id}
    >
      <span slot="header-right">
        <stencila-ui-node-execution-commands
          node-id=${this.id}
          type="CodeChunk"
        >
        </stencila-ui-node-execution-commands>
      </span>
      <div slot="body">
        <stencila-ui-node-execution-details
          type="CodeChunk"
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

        <stencila-ui-node-authors type="CodeChunk">
          <stencila-ui-node-provenance slot="provenance">
            <slot name="provenance"></slot>
          </stencila-ui-node-provenance>
          <slot name="authors"></slot>
        </stencila-ui-node-authors>

        <stencila-ui-node-code
          type="CodeChunk"
          code=${this.code}
          .code-authorship=${this.codeAuthorship}
          language=${this.programmingLanguage}
          read-only
        >
          <slot name="execution-messages" slot="execution-messages"></slot>
        </stencila-ui-node-code>
      </div>
      <div slot="content">
        ${this.isInvisible
          ? ''
          : html`
              ${this.labelType === 'TableLabel'
                ? html`<caption class="block">
                    <slot name="caption"></slot>
                  </caption>`
                : ''}
              <slot name="outputs"></slot>
              ${this.labelType === 'FigureLabel'
                ? html`<figcaption><slot name="caption"></slot></figcaption>`
                : ''}
            `}
      </div>
    </stencila-ui-block-on-demand>`
  }
}
