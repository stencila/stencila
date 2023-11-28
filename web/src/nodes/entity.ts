import { LitElement } from "lit";

export class Entity extends LitElement {
  /**
   * Override so that this element renders its content
   * in the LightDOM. This is important so that `morphdom`
   * can mutate the child content.
   */
  createRenderRoot(): HTMLElement {
    return this;
  }

  nodePatch() {
    this.dispatchEvent(new CustomEvent("node-patch", { bubbles: true }));
  }
}
