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
      <stencila-ui-node-card type="IfBlock">
        <span slot="header-right"></span>
        <div slot="body" class="h-full">
          <slot name="execution-messages"></slot>
          <slot name="authors"></slot>
        </div>
      </stencila-ui-node-card>
      <slot name="clauses"></slot>
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
