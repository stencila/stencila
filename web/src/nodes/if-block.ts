import { html } from "lit";
import { customElement } from "lit/decorators.js";

import { installTwind } from "../twind";

import { Executable } from "./executable";

const withTwind = installTwind()

/**
 * Custom element for a Stencila `IfBlock` node
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

  renderHeader() {
    return html`
      <div part="header" contenteditable="false">
        ${this.renderErrors()}
      </div>
    `
  }

  renderClauses() {
    return html`
      <div part="clauses">
        <slot name="clauses"></slot>
      </div>
    `
  }
}
