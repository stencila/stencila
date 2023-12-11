import { apply, css, tw } from "@twind/core";
import { LitElement, html } from "lit";
import { customElement, property, state } from "lit/decorators.js";
import { Ref, createRef, ref } from 'lit/directives/ref.js'

import { withTwind } from "../twind";
import type { DocumentView } from "../types";

/**
 * This class extends LitElement to include a tw instance.
 */
class TWLitElement extends LitElement {
  protected tw: typeof tw
}

/**
 * UI selector
 *
 * A selector that updates some display portion of the UI
 */
@customElement("stencila-ui-selector")
@withTwind()
export class UISelector extends TWLitElement {
  /**
   * Ref to allow us to close the details element when needed.
   */
  detailsRef: Ref<HTMLDetailsElement> = createRef();

  /**
   * Manages the open state of the open listbox
   */
  @state()
  private open: boolean = false;

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
    const open = css`
      &[open] {
        z-index: 100;
      }

      &[open] summary {
        color: ${this.tw.theme('colors.brand.blue')};
        border-bottom-color: ${this.tw.theme('colors.brand.blue')};
      }
    `

    return html`
    ${this.renderOverlay()}
    <details role="list" class="p-0 relative block flex-grow ${open}" ${ref(this.detailsRef)}>
      ${this.renderSummary()}
      <ul role="listbox" class="absolute top-8 bg-gray-aluminium flex flex-col">
        ${this.list.map(
          ([value, desc]) =>
            html`<li class="min-w-fit block whitespace-nowrap">
              <button 
                data-value="${value}"
                class="${this.target === value ? 'text-brand-red' : ''}"
                @click=${(e: Event) => {
                  this.setOpen();
                  this.clickEvent && this.clickEvent(e);
                }}
              >
              ${desc}
              </button>
            </li>`,
        )}
      </ul>
    </details>
    `
  }

  private renderSummary() {
    const styles = apply([
      "text-sm",
      "text-gray-aluminum",
      "leading-none",
      "select-none",
      "appearance-none ",
      "min-w-fit",
      "py-2 px-4",
      "bg-white",
      "border-b-2 border-b-transparent ",
      "transition-all ease-in-out ",
      "hover:text-brand-blue hover:border-b-brand-blue",
    ])

    const hideMarker = css`
      &::marker {
        display: none;
        font-size: 0;
      }
    `

    return html`<summary aria-haspopup="listbox" role="button" class="${styles} ${hideMarker}" @click=${this.setOpen}>${this.label}</summary>`
  }

  private renderOverlay() {
    return this.open ? html`<div class="w-screen h-screen fixed z-50 top-0 left-0 bg-transparent" @click=${this.setOpen}></div>` : null
  }

  private setOpen() {
    this.open = !this.open

    if (!this.open) {
      this.detailsRef.value?.removeAttribute('open')
    }
  }
}
