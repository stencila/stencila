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

  constructor(code: Cord, programmingLanguage: string, options?: Partial<CodeExpression>) {
    super(code, programmingLanguage);
    if (options) Object.assign(this, options);
    this.code = code;
    this.programmingLanguage = programmingLanguage;
  }

  /**
  * Create a `CodeExpression` from an object
  */
  static from(other: CodeExpression): CodeExpression {
    return new CodeExpression(other.code!, other.programmingLanguage!, other);
  }
}
