import { LabelType } from '@stencila/types'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { withTwind } from '../twind'
import { createCaptionLabel } from '../ui/nodes/properties/captions'

import '../ui/nodes/node-card/in-flow/block'
import '../ui/nodes/commands/execution-commands'
import '../ui/nodes/properties/authors'
import '../ui/nodes/properties/code/code'
import '../ui/nodes/properties/execution-details'
import '../ui/nodes/properties/execution-messages'
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

  override connectedCallback(): void {
    super.connectedCallback()

    createCaptionLabel(this, 'CodeChunk')
  }

  /**
   * In static view just render the outputs, label and caption
   */
  override renderStaticView() {
    return html`<div>
      <stencila-ui-node-outputs type="CodeChunk">
        <slot name="outputs"></slot>
      </stencila-ui-node-outputs>
      <div>
        <stencila-ui-node-label-and-caption
          type="CodeChunk"
          label-type=${this.labelType}
          label=${this.label}
        >
          <slot name="caption" slot="caption"></slot>
        </stencila-ui-node-label-and-caption>
      </div>
    </div>`
  }

  /**
   * In dynamic view, in addition to what is in static view, render a node card
   * with execution actions and details and code read-only and collapsed.
   */
  override renderDynamicView() {
    return html`<stencila-ui-block-on-demand
      type="CodeChunk"
      view="dynamic"
      programming-language="${this.programmingLanguage}"
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

        <stencila-ui-node-authors type="CodeChunk">
          <slot name="authors"></slot>
        </stencila-ui-node-authors>

        <stencila-ui-node-provenance type="CodeChunk">
          <slot name="provenance"></slot>
        </stencila-ui-node-provenance>

        <stencila-ui-node-code
          type="CodeChunk"
          code=${this.code}
          code-authorship=${this.codeAuthorship}
          language=${this.programmingLanguage}
          read-only
        >
          <slot name="execution-messages"></slot>
        </stencila-ui-node-code>
      </div>
      <div slot="content">
        ${this.labelType === 'TableLabel'
          ? html`<caption>
              ${this.renderCaption()}
            </caption>`
          : ''}
        ${this.isInvisible ? '' : html`<slot name="outputs"></slot>`}
        ${this.labelType === 'FigureLabel'
          ? html`<figcaption>${this.renderCaption()}</figcaption>`
          : ''}
      </div>
    </stencila-ui-block-on-demand>`
  }

  private renderCaption() {
    return html`<slot name="caption"></slot>`
  }

  /**
   * In source view render everything as in dynamic view except for
   * code, label, caption (because they are displayed in the source code).
   */
  override renderSourceView() {
    return html`<stencila-ui-block-in-flow type="CodeChunk" view="source">
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

        <stencila-ui-node-authors type="CodeChunk">
          <slot name="authors"></slot>
        </stencila-ui-node-authors>

        <stencila-ui-node-execution-messages
          type="CodeChunk"
          message-count=${this.messageCount}
          warning-count=${this.warningCount}
          error-count=${this.errorCount}
        >
          <slot name="execution-messages"></slot>
        </stencila-ui-node-execution-messages>

        <stencila-ui-node-outputs type="CodeChunk">
          <slot name="outputs"></slot>
        </stencila-ui-node-outputs>
      </div>
    </stencila-ui-block-in-flow>`
  }
}
