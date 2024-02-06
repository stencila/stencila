// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { CodeExecutable } from "./CodeExecutable.js";
import { Cord } from "./Cord.js";
import { Node } from "./Node.js";

/**
 * An executable programming code expression.
 */
export class CodeExpression extends CodeExecutable {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "CodeExpression";

  /**
   * The value of the expression when it was last evaluated.
   */
  output?: Node;

  constructor(code: Cord, options?: Partial<CodeExpression>) {
    super(code);
    this.type = "CodeExpression";
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
