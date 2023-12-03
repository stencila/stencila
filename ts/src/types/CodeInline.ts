// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CodeStatic } from "./CodeStatic.js";
import { Cord } from "./Cord.js";

/**
 * Inline code.
 */
export class CodeInline extends CodeStatic {
  type = "CodeInline";

  constructor(code: Cord, options?: Partial<CodeInline>) {
    super(code);
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
