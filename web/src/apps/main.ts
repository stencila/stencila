import { LitElement, html } from "lit";
import { customElement, state } from "lit/decorators.js";

import logo from "../images/stencilaIcon.svg";
import { installTwind } from "../twind";

// Include all node components required for this view
import "../views/split";
import "../views/live";
import "../views/source";
import "../views/dynamic";
// import "../views/print";

import "./main.css";

type View = "dynamic" | "live" | "source" | "split" | "visual";

/**
 * Application Wrapper
 *
 * Wraps the application in the `app-chrome`. Contains the main header and
 * footer.
 */
@customElement("stencila-main-app")
@installTwind()
export class App extends LitElement {
  @state()
  private view: View = "live";

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
              here ${this.renderView()}
            </div>
          </main>
          ${this.renderFooter()}
        </div>
      </div>
    `;
  }

  // private _changeViewSelector(e: Event) {
  //   this.view = (e.target as HTMLSelectElement).value as View
  // }

  private renderHeader() {
    return html`<header
      class="sticky w-full top-0 left-0 z-30 h-16 drop-shadow-[0_2px_0_#edf2f7] border-t-[3px] bg-white border-t-brand-blue p-4"
    >
      <nav class="container mx-auto flex justify-items-center">
        <a href="/"
          ><img src="${logo}" alt="Stencila logo" width="28" height="28"
        /></a>
      </nav>
    </header>`;
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
      case "live":
        return html`<stencila-live-view></stencila-live-view>`;
      default:
        return html`<stencila-dynamic-view></stencila-dynamic-view>`;
    }
  }
}
