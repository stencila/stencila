import { Node } from "@stencila/types";
import { customElement, property } from "lit/decorators.js";

import { CodeExecutable } from "./code-executable";

@customElement("stencila-code-expression")
export class CodeExpression extends CodeExecutable {
  @property()
  output?: Node;
}
