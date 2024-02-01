// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { CodeLocation } from "./CodeLocation.js";
import { Entity } from "./Entity.js";

/**
 * An error that occurred when executing an executable node.
 */
export class ExecutionError extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ExecutionError";

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

  /**
   * Stack trace leading up to the error.
   */
  stackTrace?: string;

  constructor(errorMessage: string, options?: Partial<ExecutionError>) {
    super();
    this.type = "ExecutionError";
    if (options) Object.assign(this, options);
    this.errorMessage = errorMessage;
  }
}

/**
* Create a new `ExecutionError`
*/
export function executionError(errorMessage: string, options?: Partial<ExecutionError>): ExecutionError {
  return new ExecutionError(errorMessage, options);
}
