// Generated file; do not edit. See `../rust/schema-gen` crate.

// An error that occurred when parsing, compiling or executing a Code node.
export class CodeError {
  type = "CodeError";

  // The identifier for this item
  id?: string;

  // The error message or brief description of the error.
  errorMessage: string;

  // The type of error e.g. "SyntaxError", "ZeroDivisionError".
  errorType?: string;

  // Stack trace leading up to the error.
  stackTrace?: string;

  constructor(errorMessage: string, options?: CodeError) {
    if (options) Object.assign(this, options)
    this.errorMessage = errorMessage;
  }
}
