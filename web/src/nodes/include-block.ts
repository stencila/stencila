import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import { Executable } from './executable'
import { nodeCardStyles } from './helpers/node-card'

/**
 * Web component representing a Stencila Schema `IncludeBlock` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/include-block.md
 */
@customElement('stencila-include-block')
export class IncludeBlock extends Executable {
  /**
   * path of the file being 'included'
   */
  @property({ type: String })
  source: string

  // TODO: render the source field properly, currently using placeholder

  override renderStaticView() {
    return html`
      <stencila-node-card type="InlcudeBlock">
        <div slot="body"><span>source: </span><span>${this.source}</span></div>
      </stencila-node-card>
    `
  }

  override renderDynamicView() {
    return html`
      <stencila-node-card type="InlcudeBlock">
        <div slot="body"><span>source: </span><span>${this.source}</span></div>
        <slot name="authors"></slot>
        <slot name="output"></slot>
      </stencila-node-card>
    `
  }

  override renderVisualView() {
    return this.renderDynamicView()
  }

  override renderSourceView() {
    return html`
      <stencila-node-card
        type="InlcudeBlock"
        class=${nodeCardStyles(this.documentView())}
      >
        <div slot="body">icon</span><span>${this.source}</span></div>
        <slot name="authors"></slot>
      </stencila-node-card>
    `
  }
}
