// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CodeExecutable } from "./CodeExecutable.js";
import { Cord } from "./Cord.js";
import { Node } from "./Node.js";

/**
 * An executable programming code expression.
 */
export class CodeExpression extends CodeExecutable {
  type = "CodeExpression";

  /**
   * The value of the expression when it was last evaluated.
   */
  output?: Node;

  constructor(code: Cord, options?: Partial<CodeExpression>) {
    super(code);
    if (options) Object.assign(this, options);
    this.code = code;
  }
}

/**
* Create a new `CodeExpression`
*/
export function codeExpression(code: Cord, options?: Partial<CodeExpression>): CodeExpression {
  return new CodeExpression(code, options);
}
