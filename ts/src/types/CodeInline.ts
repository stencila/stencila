// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { CodeStatic } from "./CodeStatic.js";
import { Cord } from "./Cord.js";

/**
 * Inline code.
 */
export class CodeInline extends CodeStatic {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "CodeInline";

  constructor(code: Cord, options?: Partial<CodeInline>) {
    super(code);
    this.type = "CodeInline";
    if (options) Object.assign(this, options);
    this.code = code;
  }
}

/**
* Create a new `CodeInline`
*/
export function codeInline(code: Cord, options?: Partial<CodeInline>): CodeInline {
  return new CodeInline(code, options);
}
