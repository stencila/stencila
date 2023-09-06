// Generated file; do not edit. See `../rust/schema-gen` crate.

import { ExecutionDigest } from './ExecutionDigest';

// A fragment of math, e.g a variable name, to be treated as inline content.
export class MathFragment {
  type = "MathFragment";

  // The identifier for this item
  id?: string;

  // The language used for the equation e.g tex, mathml, asciimath.
  mathLanguage: string;

  // The code of the equation in the `mathLanguage`.
  code: string;

  // A digest of the `code` and `mathLanguage`.
  compileDigest?: ExecutionDigest;

  // Errors that occurred when parsing the math equation.
  errors?: string[];

  // The MathML transpiled from the `code`.
  mathml?: string;

  constructor(mathLanguage: string, code: string, options?: MathFragment) {
    if (options) Object.assign(this, options)
    this.mathLanguage = mathLanguage;
    this.code = code;
  }
}
