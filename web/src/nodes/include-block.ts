import { customElement } from "lit/decorators.js";

import { Executable } from "./executable";

/**
 * Web component representing a Stencila Schema `IncludeBlock` node
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/include-block.md
 */
@customElement("stencila-include-block")
export class IncludeBlock extends Executable {}
