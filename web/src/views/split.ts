import { LitElement, html } from "lit";
import { customElement, property } from "lit/decorators";

import { DocumentAccess } from "../types";

import "./dynamic";
import "./source";

/**
 * Split panes view for a document
 */
@customElement("stencila-split-view")
export class SplitView extends LitElement {
  /**
   * The access level of the view
   *
   * Passed through to child views.
   */
  @property()
  access: DocumentAccess = "code";

  /**
   * The format of the source code editor
   */
  @property()
  format: string;

  /**
   * Override to use the Light DOM which is necessary for the `Dynamic` views.
   */
  override createRenderRoot() {
    return this;
  }

  protected render() {
    return html`
      <stencila-source-view
        view="source"
        id=${this.id}
        access=${this.access}
        format=${this.format}
      ></stencila-source-view>

      <stencila-dynamic-view view="dynamic" id=${this.id} access=${this.access}>
        <article data-root></article>
      </stencila-dynamic-view>
    `;
  }
}
