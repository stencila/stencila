// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { CodeLocation } from "./CodeLocation.js";
import { Entity } from "./Entity.js";

/**
 * An error that occurred while compiling an executable node.
 */
export class CompilationError extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "CompilationError";

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
    this.type = "CompilationError";
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
