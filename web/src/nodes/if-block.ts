import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { Executable } from './executable'
import { nodeCardStyles } from './helpers/node-card'

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
      <stencila-node-card
        type="IfBlock"
        class=${nodeCardStyles(this.documentView())}
      >
        <span slot="header-right">${this.renderExecutableButtons()}</span>
        <div slot="body" class="h-full">
          <slot name="execution-messages"></slot>
        </div>
      </stencila-node-card>
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
      <stencila-node-card
        type="IfBlock"
        class=${nodeCardStyles(this.documentView())}
      >
        <span slot="header-right">${this.renderExecutableButtons()}</span>
        <div slot="body" class="h-full">
          <slot name="execution-messages"></slot>
          <slot name="authors"></slot>
        </div>
      </stencila-node-card>
    `
  }

  // ${this.renderTimeFields()}
  // private renderClauses() {
  //   return html`
  //     <div>
  //       <slot name="clauses"></slot>
  //     </div>
  //   `
  // }
}
