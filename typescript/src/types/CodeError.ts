// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";

/**
 * An error that occurred when parsing, compiling or executing a `Code` node.
 */
export class CodeError extends Entity {
  type = "CodeError";

  /**
   * The error message or brief description of the error.
   */
  errorMessage: string;

  /**
   * The type of error e.g. "SyntaxError", "ZeroDivisionError".
   */
  errorType?: string;

  /**
   * Stack trace leading up to the error.
   */
  stackTrace?: string;

  constructor(errorMessage: string, options?: Partial<CodeError>) {
    super();
    if (options) Object.assign(this, options);
    this.errorMessage = errorMessage;
  }
}

/**
* Create a new `CodeError`
*/
export function codeError(errorMessage: string, options?: Partial<CodeError>): CodeError {
  return new CodeError(errorMessage, options);
}
