import { css, tw, tx } from "@twind/core";
import { LitElement, html } from "lit";
import { customElement, property } from "lit/decorators.js";

import { withTwind } from "../twind";
import type { DocumentView } from "../types";

// import "./selector-styles.css";

/**
 * UI selector
 *
 * A selector that updates some display portion of the UI
 */
@customElement("stencila-ui-selector")
@withTwind()
export class UISelector extends LitElement {
  /**
   * Manages the open state of the open listbox
   */
  open: boolean = false;

  /**
   * Label displayed when listbox is not open
   */
  @property()
  label: string = "";

  /**
   * List of values to render in the list
   */
  @property({ type: Array })
  list: [string, string][] = [];

  /**
   * Event to call when a list element is selected
   */
  @property()
  clickEvent: (e: Event) => void | undefined;

  /**
   * Target property in parent component to evaluate
   */
  @property()
  target: DocumentView | string
 

  override render() {
    // const x = tx(`border-b-brand-blue`)
    const open = css`
      &[open] summary {
        border-bottom-color: ${this.tw.theme`colors.brand.blue`};
      }
    `

    return html`
    <details role="list" class="p-0 relative block flex-grow ${open}">
      ${this.renderSummary()}
      <ul role="listbox" class="absolute top-8 bg-gray-aluminium flex flex-col">
        ${this.list.map(
          ([value, desc]) =>
            html`<li class="min-w-fit block whitespace-nowrap">
              <button data-value="${value}" class="${this.target === value ? 'text-brand-red' : ''}" @click=${(e: Event) => {
      this.open = !this.open;
      this.clickEvent && this.clickEvent(e);
    }}>
              ${desc}
              </button>
            </li>`,
        )}
      </ul>
    </details>
    `
  }

  private renderSummary() {
    const hideMarker = css`
      &::marker {
        display: none;
        font-size: 0;
      }
    `

    return html`<summary aria-haspopup="listbox" role="button" class="min-w-fit select-none bg-white leading-none text-sm py-2 px-4 text-gray-aluminum appearance-none ${hideMarker} border-b-2 border-b-transparent transition-all ease-in-out hover:text-brand-blue hover:border-b-brand-blue" @click=${() => { this.open = !this.open; console.log(this.open) }}>${this.label}</summary>`
  }
}
