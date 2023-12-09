import { LitElement, html } from "lit";
import { customElement, property, state } from "lit/decorators.js";

import { installTwind } from "../twind";

// Include all node components required for this view
import "../views/split";
import "../views/dynamic";

type View = "dynamic" | "live" | "source" | "static" | "visual";

/**
 * Application Wrapper
 *
 * Wraps the application in the `app-chrome`. Contains the main header and
 * footer.
 */
@customElement("stencila-app-view")
@installTwind()
export class App extends LitElement {
  @property({ type: String })
  version = "dev";

  @state()
  private view: View = "static";

  override render() {
    return html`
      ${this.renderHeader()}
      <div class="h-screen flex flex-col">
        <main role="main" class="flex-grow">${this.renderView()}</main>
        <footer>Footer</footer>
      </div>
    `;
  }

  private _changeViewSelector(e: Event) {
    this.view = (e.target as HTMLSelectElement).value as View;
  }

  private renderHeader() {
    return html`<header
      class="fixed w-full top-0 left-0 z-30 h-16 drop-shadow-[0_2px_0_#edf2f7] border-t-[3px] bg-white border-t-brand-blue p-4"
    >
      <nav class="container mx-auto flex justify-items-center">
        <a href="/"
          ><img
            src="/~static/${this.version}/images/stencilaIcon.svg"
            alt="Stencila logo"
            width="28"
            height="28"
        /></a>
        <select @change=${this._changeViewSelector}>
          <option value="dynamic">dynamic</option>
          <option value="static">static</option>
        </select>
      </nav>
    </header>`;
  }

  private renderView() {
    switch (this.view) {
      case "dynamic":
        return html`<stencila-dynamic-view
          ><slot></slot
        ></stencila-dynamic-view>`;
      case "static":
        return html`<stencila-static-view><slot></slot></stencila-static-view>`;
    }
  }
}
