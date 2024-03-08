import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

import './widgets/basic-field'
import './widgets/collapsable-field'
import './helpers/node-card'
import { withTwind } from '../twind'

import { CodeExecutable } from './code-executable'

/**
 * Web component representing a Stencila Schema `CodeChunk` node
 *
 * @slot outputs
 * @slot caption
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-chunk.md
 */
@customElement('stencila-code-chunk')
@withTwind()
export class CodeChunk extends CodeExecutable {
  @property()
  label?: string

  /**
   * In static view just render the outputs, label and caption
   */
  override renderStaticView() {
    return html`<div>
      <slot name="outputs"></slot>
      <div>
        ${this.renderLabel()}
        <slot name="caption"></slot>
      </div>
    </div>`
  }

  /**
   * In dynamic view, in addition to what is in static view,
   * render header with details such as status and action buttons,
   * and code read-only and default collapsed.
   */
  override renderDynamicView() {
    return html`<stencila-node-card type="CodeChunk">
      <span slot="header-right">${this.renderExecutableButtons()}</span>
      <div slot="body">
        ${this.renderTimeFields()}
        <!-- TODO: readonly codemirror editor -->
        <slot name="code"></slot>
        <slot name="outputs"></slot>
        <slot name="execution-messages"></slot>
        <div>
          ${this.renderLabel()}
          <slot name="caption"></slot>
        </div>
      </div>
    </stencila-node-card>`
  }

  /**
   * In source view render everything as in dynamic view except for
   * code, label, caption (because they are editable in the editor).
   */
  override renderSourceView() {
    const nodeCardStyles = ['flex flex-col', 'h-full']

    return html`<stencila-node-card type="CodeChunk" class=${nodeCardStyles}>
      <span slot="header-right">${this.renderExecutableButtons()}</span>
      <div slot="body" class="h-full">
        ${this.renderTimeFields()}
        <slot name="execution-messages"></slot>
        <slot name="outputs"></slot>
      </div>
    </stencila-node-card>`
  }

  private renderLabel() {
    return this.label ? html`<span>${this.label}</span>` : ''
  }
}
