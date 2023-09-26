// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CodeStatic } from "./CodeStatic.js";
import { Cord } from "./Cord.js";

/**
 * A code block.
 */
export class CodeBlock extends CodeStatic {
  type = "CodeBlock";

  constructor(code: Cord, options?: Partial<CodeBlock>) {
    super(code);
    if (options) Object.assign(this, options);
    this.code = code;
  }
}

/**
* Create a new `CodeBlock`
*/
export function codeBlock(code: Cord, options?: Partial<CodeBlock>): CodeBlock {
  return new CodeBlock(code, options);
}
