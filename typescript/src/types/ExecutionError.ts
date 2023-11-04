// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Entity } from "./Entity.js";

/**
 * An error that occurred when executing an `Executable` node.
 */
export class ExecutionError extends Entity {
  type = "ExecutionError";

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

  constructor(errorMessage: string, options?: Partial<ExecutionError>) {
    super();
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
