// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Cord } from "./Cord.js";
import { Entity } from "./Entity.js";

/**
 * Abstract base type for non-executable code nodes (e.g. `CodeBlock`).
 */
export class CodeStatic extends Entity {
  type = "CodeStatic";

  /**
   * The code.
   */
  code: Cord;

  /**
   * The programming language of the code.
   */
  programmingLanguage?: string;

  constructor(code: Cord, options?: Partial<CodeStatic>) {
    super();
    if (options) Object.assign(this, options);
    this.code = code;
  }
}

/**
* Create a new `CodeStatic`
*/
export function codeStatic(code: Cord, options?: Partial<CodeStatic>): CodeStatic {
  return new CodeStatic(code, options);
}
