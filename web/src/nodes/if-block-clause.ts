import { html } from "lit";
import { customElement, state } from "lit/decorators.js";

import { installTwind } from "../twind";

import { CodeExecutable } from "./code-executable";

const withTwind = installTwind();

/**
 * Custom element for a Stencila `IfBlockClause` node
 */
@customElement("stencila-if-block-clause")
@withTwind
export class IfBlockClause extends CodeExecutable {
  /**
   * Whether the clause has any content
   *
   * This state is used to determine whether to render placeholder
   * text if there is no content
   *
   * @see this.renderContent()
   */
  @state()
  private hasContent = false;

  /**
   * A mutation observer to update the `hasContent` state when
   * the `content` slot changes
   */
  private contentObserver: MutationObserver;

  /**
   * Handle a change, including on initial load, of the `content` slot
   */
  onContentSlotChange(event: Event) {
    // Get the slot element
    const contentElem = (event.target as HTMLSlotElement).assignedElements({
      flatten: true,
    })[0];

    // Set current state
    this.hasContent = contentElem.childElementCount > 0;

    // Update the state when the slot is mutated
    this.contentObserver = new MutationObserver(() => {
      this.hasContent = contentElem.childElementCount > 0;
    });
    this.contentObserver.observe(contentElem, {
      childList: true,
    });
  }

  override render() {
    return html`
      <div part="root" class="border-(1 rose-200) p-2">
        ${this.renderHeader()} ${this.renderContent()}
      </div>
    `;
  }

  renderHeader() {
    return html`
      <div part="header" contenteditable="false">${this.renderErrors()}</div>
    `;
  }

  renderContent() {
    return html`
      <div part="content">
        <p class="text-gray-400" contenteditable="false">
          ${this.hasContent ? "" : "No content"}
        </p>
        <slot
          name="content"
          @slotchange=${this.onContentSlotChange}
        ></slot>
      </div>
    `;
  }
}
