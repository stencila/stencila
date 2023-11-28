import { customElement } from "lit/decorators.js";

import { Executable } from "./executable";

@customElement("stencila-parameter")
export class Parameter extends Executable {
  constructor() {
    super();

    this.addEventListener("input", (event: Event) => {
      console.log(event)

      //this.nodePatch()
    });
  }
}
