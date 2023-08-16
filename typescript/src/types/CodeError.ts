// Generated file. Do not edit; see `rust/schema-gen` crate.

import { String } from './String';

// An error that occurred when parsing, compiling or executing a Code node.
export class CodeError {
  // The type of this item
  type = "CodeError";

  // The identifier for this item
  id?: String;

  // The error message or brief description of the error.
  errorMessage: String;

  // The type of error e.g. "SyntaxError", "ZeroDivisionError".
  errorType?: String;

  // Stack trace leading up to the error.
  stackTrace?: String;

  constructor(errorMessage: String, options?: CodeError) {
    if (options) Object.assign(this, options)
    this.errorMessage = errorMessage;
  }
}
