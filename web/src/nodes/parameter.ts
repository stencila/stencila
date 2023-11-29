import type { Validator, Node } from "@stencila/types";
import { html } from "lit";
import { customElement, property } from "lit/decorators.js";

import { Executable } from "./executable";

@customElement("stencila-parameter")
export class Parameter extends Executable {
  @property()
  name: string

  @property()
  label?: string

  @property()
  value?: Node

  @property()
  default?: Node

  @property()
  validator?: Validator

  constructor() {
    super();

    this.addEventListener("input", (event: Event) => {
      const target = event.target as HTMLInputElement;

      const value = target.value

      // TODO: Handle different types of values
      // using target.valueAsNumber and target.valueAsDate

      this.patchNode({
        op: 'replace',
        id: this.id,
        path: "value",
        value,
      })
    });
  }

  render() {
    return html`
      <label for="${this.name}">${this.label ?? this.name}</label>
      <input name="${this.name}">
    `
  }
}
