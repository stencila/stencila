import { html } from 'lit'
import { customElement } from 'lit/decorators.js'

import { withTwind } from '../twind'

import { CodeExecutable } from './code-executable'
import { nodeCardStyles } from './helpers/node-card'

/**
 * Web component representing a Stencila Schema `For` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/for-block.md
 */
@customElement('stencila-for-block')
@withTwind()
export class ForBlock extends CodeExecutable {
  override renderStaticView() {
    return html`<stencila-node-card type="ForBlock">
        <div slot="body" class="h-full">
          <slot name="code"></slot>
        </div>
      </stencila-node-card>
      <slot name="iterations"></slot>`
  }

  override renderDynamicView() {
    return html`
      <stencila-node-card type="ForBlock">
        <span slot="header-right">${this.renderExecutableButtons()}</span>
        <div slot="body" class="h-full">
          <slot name="code"></slot>
          ${this.renderTimeFields()}
          <slot name="execution-messages"></slot>
        </div>
      </stencila-node-card>
      <slot name="iterations"></slot>
    `
  }

  override renderVisualView() {
    return this.renderDynamicView()
  }

  override renderSourceView() {
    return html`
      <stencila-node-card
        type="ForBlock"
        class=${nodeCardStyles(this.documentView())}
      >
        <span slot="header-right">${this.renderExecutableButtons()}</span>
        <div slot="body" class="h-full">
          ${this.renderTimeFields()}
          <slot name="execution-messages"></slot>
        </div>
      </stencila-node-card>
    `
  }
}
