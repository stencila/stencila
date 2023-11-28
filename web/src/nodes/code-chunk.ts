import { customElement } from "lit/decorators.js";

import { CodeExecutable } from "./code-executable";

@customElement("stencila-code-chunk")
export class CodeChunk extends CodeExecutable {}
