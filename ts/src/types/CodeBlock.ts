// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { CodeStatic } from "./CodeStatic.js";
import { Cord } from "./Cord.js";

/**
 * A code block.
 */
export class CodeBlock extends CodeStatic {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "CodeBlock";

  constructor(code: Cord, options?: Partial<CodeBlock>) {
    super(code);
    this.type = "CodeBlock";
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
