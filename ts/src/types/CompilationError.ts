// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CodeLocation } from "./CodeLocation.js";
import { Entity } from "./Entity.js";

/**
 * An error that occurred while compiling an executable node.
 */
export class CompilationError extends Entity {
  type = "CompilationError";

  /**
   * The error message or brief description of the error.
   */
  errorMessage: string;

  /**
   * The type of error e.g. "SyntaxError", "ZeroDivisionError".
   */
  errorType?: string;

  /**
   * The location that the error occurred.
   */
  codeLocation?: CodeLocation;

  constructor(errorMessage: string, options?: Partial<CompilationError>) {
    super();
    if (options) Object.assign(this, options);
    this.errorMessage = errorMessage;
  }
}

/**
* Create a new `CompilationError`
*/
export function compilationError(errorMessage: string, options?: Partial<CompilationError>): CompilationError {
  return new CompilationError(errorMessage, options);
}
