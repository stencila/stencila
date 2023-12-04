import { LitElement } from "lit";
import { customElement, property } from "lit/decorators.js";

import { DomClient } from "../clients/dom";
import { NodesClient } from "../clients/nodes";
import { type DocumentAccess } from "../types";

// Include all node components required for this view
import "../nodes/code-chunk";
import "../nodes/code-expression";
import "../nodes/if-block";
import "../nodes/if-block-clause";
import "../nodes/parameter";

import "./dynamic.css";

/**
 * Dynamic view of a document
 *
 * A view which, in addition to providing live updates of a document,
 * allows for the user to change input values (e.g. the `value` of a `Parameter` node)
 */
@customElement("stencila-dynamic")
export class Dynamic extends LitElement {
  /**
   * The access level of the view
   *
   * This property is passed through to the `NodesClient` but may also
   * be inspected by descendent WebComponents to determine their behavior.
   *
   * This should not be `edit`, `write` or `admin` since this view
   * does not provide the means to modify those.
   */
  @property()
  access: DocumentAccess = "code";

  /**
   * A read-only client which updates the document's DOM when the
   * document changes on the server
   */
  private domClient: DomClient;

  /**
   * A write-only client which sends node patches to the document
   * on the server
   */
  private nodesClient: NodesClient;

  /**
   * Override so that the document's DOM is rendered in the Light DOM
   * which is necessary for the `domClient` to work.
   *
   * @override
   */
  createRenderRoot(): HTMLElement {
    return this;
  }

  /**
   * Override so that the clients ae instantiated _after_ this
   * element has a `renderRoot`.
   *
   * @override
   */
  connectedCallback() {
    super.connectedCallback();

    this.domClient = new DomClient(
      this.id,
      this.renderRoot.firstElementChild as HTMLElement
    );

    this.nodesClient = new NodesClient(
      this.id,
      this.access,
      this.renderRoot as HTMLElement
    );
  }
}
