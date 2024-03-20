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
  override renderStaticView() {
    return html`<stencila-ui-node-card type="ForBlock">
        <div slot="body" class="h-full">
          <slot name="code"></slot>
        </div>
      </stencila-ui-node-card>
      <slot name="iterations"></slot>`
  }

  override renderDynamicView() {
    return html`
      <stencila-ui-node-card type="ForBlock" view="dynamic">
        <span slot="header-right">
          <stencila-ui-node-execution-commands node-id=${this.id}>
          </stencila-ui-node-execution-commands>
        </span>
        <div slot="body" class="h-full">
          <slot name="code"></slot>
          <slot name="execution-messages"></slot>
        </div>
      </stencila-ui-node-card>
      <slot name="iterations"></slot>
    `
  }

  override renderSourceView() {
    return html`
      <stencila-ui-node-card type="ForBlock" view="source">
        <span slot="header-right">
          <stencila-ui-node-execution-commands node-id=${this.id}>
          </stencila-ui-node-execution-commands>
        </span>
        <div slot="body" class="h-full">
          <slot name="execution-messages"></slot>
        </div>
      </stencila-ui-node-card>
    `
  }
}
