// Generated file; do not edit. See `../rust/schema-gen` crate.

import { ExecutionDigest } from './ExecutionDigest';
import { String } from './String';

// A fragment of math, e.g a variable name, to be treated as inline content.
export class MathFragment {
  // The type of this item
  type = "MathFragment";

  // The identifier for this item
  id?: String;

  // The language used for the equation e.g tex, mathml, asciimath.
  mathLanguage: String;

  // The code of the equation in the `mathLanguage`.
  code: String;

  // A digest of the `code` and `mathLanguage` used to avoid unnecessary transpilation to MathML
  compileDigest?: ExecutionDigest;

  // Errors that occurred when parsing the math equation.
  errors?: String[];

  // The MathML transpiled from the `code`
  mathml?: String;

  constructor(mathLanguage: String, code: String, options?: MathFragment) {
    if (options) Object.assign(this, options)
    this.mathLanguage = mathLanguage;
    this.code = code;
  }
}
