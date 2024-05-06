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
  override renderStaticView() {
    return html`<div></div>`
  }

  override renderDynamicView() {
    return html`
      <stencila-ui-block-on-demand type="IfBlock">
        <span slot="header-right"></span>
        <div slot="body" class="h-full">
          <stencila-ui-node-execution-details
            type="IfBlock"
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

          <stencila-ui-node-authors type="IfBlock">
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

  override renderVisualView() {
    return this.renderDynamicView()
  }

  override renderInteractiveView() {
    return this.renderDynamicView()
  }

  override renderSourceView() {
    return html`
      <stencila-ui-node-card type="IfBlock">
        <span slot="header-right"></span>
        <div slot="body" class="h-full">
          <slot name="execution-messages"></slot>
          <slot name="authors"></slot>
        </div>
      </stencila-ui-node-card>
    `
  }
}
