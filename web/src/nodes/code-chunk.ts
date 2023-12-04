import { html } from "lit";
import { customElement, property } from "lit/decorators.js";

import { CodeExecutable } from "./code-executable";

/**
 * Web component representing a Stencila Schema `CodeChunk` node
 *
 * @slot outputs
 * @slot caption
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/code/code-chunk.md
 */
@customElement("stencila-code-chunk")
export class CodeChunk extends CodeExecutable {
  @property()
  label?: string;

  override render() {
    return html`<span>
      <slot name="outputs"></slot>
      ${this.renderLabel()}
      <slot name="caption"></slot>
    </span>`;
  }

  private renderLabel() {
    return this.label ? html`<span part="label">${this.label}</span>` : "";
  }
}
