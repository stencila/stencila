import { LitElement } from "lit";
import { customElement } from "lit/decorators.js";

import { DomClient } from "../clients/dom";

/**
 * Live view of a document
 *
 * A view which provides live updates of a document's DOM as it changes
 * on the server.
 */
@customElement("stencila-live-view")
export class LiveView extends LitElement {
  /**
   * A read-only client which will update the document's DOM when the
   * document changes
   */
  private domClient: DomClient;

  /**
   * Override so that the document's DOM is rendered in the Light DOM
   * which is necessary for the `domClient` to work.
   */
  override createRenderRoot(): HTMLElement {
    return this;
  }

  /**
   * Override so that the `domClient` is instantiated _after_ this
   * element has a `renderRoot`.
   */
  override connectedCallback() {
    super.connectedCallback();

    this.domClient = new DomClient(
      this.id,
      this.renderRoot.firstElementChild as HTMLElement
    );
  }
}
