import { LitElement, html } from "lit";
import { customElement, property } from "lit/decorators.js";

import logo from "../images/stencilaIcon.svg";
import { THEMES } from "../themes/themes";
import { installTwind } from "../twind";
import type { DocumentId, DocumentView } from "../types";
import { VIEWS } from "../views/views";

import "../views/split";
import "../views/live";
import "../views/source";
import "../views/dynamic";
// import "../views/print";

import "./main.css";

/**
 * Application Wrapper
 *
 * Wraps the application in the `app-chrome`. Contains the main header and
 * footer.
 */
@customElement("stencila-main-app")
@installTwind()
export class App extends LitElement {
  /**
   * The id of the current document (if any)
   *
   * The app can be opened with a document or not. If there is no `doc` attribute
   * (e.g. because the server could not resolve a file from the URL path)
   * then the app should offer some suggestions.
   */
  @property()
  doc?: DocumentId;

  /**
   * The current view of the current document
   *
   * If there is no `view` attribute then this will be dynamically
   * determined based on the maximum access level that the user has for
   * the document.
   */
  @property()
  view?: DocumentView = "live";

  /**
   * The theme to use for HTML-based views of the document (e.g. `static`, `live`)
   */
  @property()
  theme: string = "default";

  /**
   * The format to use for source views of the document (`source` and `split` view)
   */
  @property()
  format: string = "markdown";

  override render() {
    return html`
      <div class="font-sans">
        ${this.renderHeader()}
        <div class="h-screen flex flex-col">
          <main
            role="main"
            class="flex-grow px-4 py-8 w-full justify-center flex"
          >
            <div
              class="bg-white border border-grays-mid container p-4 mx-0 h-full shadow-[0_0_8px_rgba(0,0,0,.035)]"
            >
              ${this.doc ? this.renderView() : "No document specified"}
            </div>
          </main>
          ${this.renderFooter()}
        </div>
      </div>
    `;
  }

  private renderHeader() {
    return html`<header
      class="sticky w-full top-0 left-0 z-30 h-16 drop-shadow-[0_2px_0_#edf2f7] border-t-[3px] bg-white border-t-brand-blue p-4"
    >
      <nav class="container mx-auto flex justify-items-center">
        <a href="/"
          ><img src="${logo}" alt="Stencila logo" width="28" height="28"
        /></a>
      </nav>
      ${this.renderViewSelect()} ${this.renderThemeSelect()}
    </header>`;
  }

  private renderViewSelect() {
    return html`
      <label>
        View
        <select
          @change=${(e: Event) =>
            (this.view = (e.target as HTMLSelectElement).value as DocumentView)}
        >
          ${Object.entries(VIEWS).map(
            ([view, desc]) =>
              html`<option
                value=${view}
                title=${desc}
                ?selected=${this.view === view}
              >
                ${view}
              </option>`,
          )}
        </select>
      </label>
    `;
  }

  private renderThemeSelect() {
    return html`
      <label>
        Theme
        <select
          @change=${(e: Event) =>
            (this.theme = (e.target as HTMLSelectElement).value)}
        >
          ${Object.entries(THEMES).map(
            ([theme, desc]) =>
              html`<option
                value=${theme}
                title=${desc}
                ?selected=${this.theme === theme}
              >
                ${theme}
              </option>`,
          )}
        </select>
      </label>
    `;
  }

  private renderFooter() {
    return html`<footer class="bg-brand-blue px-4 py-6 text-white">
      <div class="container mx-auto">
        <p class="text-sm my-0">&copy; 2023 Stencila Ltd.</p>
      </div>
    </footer>`;
  }

  private renderView() {
    switch (this.view) {
      case "static":
        return html`<stencila-static-view
          view="static"
          doc=${this.doc}
          theme=${this.theme}
        ></stencila-static-view>`;

      case "print":
        return html`<stencila-print-view
          view="print"
          doc=${this.doc}
          theme=${this.theme}
        ></stencila-print-view>`;

      case "live":
        return html`<stencila-live-view
          view="live"
          doc=${this.doc}
          theme=${this.theme}
        ></stencila-live-view>`;

      case "dynamic":
        return html`<stencila-dynamic-view
          view="dynamic"
          doc=${this.doc}
          theme=${this.theme}
        ></stencila-dynamic-view>`;

      case "source":
        return html`<stencila-source-view
          view="source"
          doc=${this.doc}
          format=${this.format}
        ></stencila-source-view>`;

      case "split":
        return html`<stencila-split-view
          view="split"
          doc=${this.doc}
          format=${this.format}
          theme=${this.theme}
        ></stencila-split-view>`;

      case "visual":
        return html`<stencila-visual-view
          view="visual"
          doc=${this.doc}
          theme=${this.theme}
        ></stencila-visual-view>`;
    }
  }
}
