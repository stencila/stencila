import { html } from "lit";
import { customElement } from "lit/decorators.js";

import { installTwind } from "../twind";

import { Executable } from "./executable";

const withTwind = installTwind()

/**
 * Web component representing a Stencila Schema `IfBlock` node
 * 
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/if-block.md
 */
@customElement("stencila-if-block")
@withTwind
export class IfBlock extends Executable {
  override render() {
    return html`
      <div part="root" class="border-(1 rose-500) p-2">
        ${this.renderHeader()}
        ${this.renderClauses()}
      </div>
    `
  }

  private renderHeader() {
    return html`
      <div part="header" contenteditable="false">
        ${this.renderErrors()}
      </div>
    `
  }

  private renderClauses() {
    return html`
      <div part="clauses">
        <slot name="clauses"></slot>
      </div>
    `
  }
}
