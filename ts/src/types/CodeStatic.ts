// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Author } from "./Author.js";
import { Cord } from "./Cord.js";
import { Entity } from "./Entity.js";

/**
 * Abstract base type for non-executable code nodes (e.g. `CodeBlock`).
 */
export class CodeStatic extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "CodeStatic";

  /**
   * The code.
   */
  code: Cord;

  /**
   * The programming language of the code.
   */
  programmingLanguage?: string;

  /**
   * The authors of the code.
   */
  authors?: Author[];

  constructor(code: Cord, options?: Partial<CodeStatic>) {
    super();
    this.type = "CodeStatic";
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
