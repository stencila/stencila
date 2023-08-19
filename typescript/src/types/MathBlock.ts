// Generated file; do not edit. See `../rust/schema-gen` crate.

import { ExecutionDigest } from './ExecutionDigest';

// A block of math, e.g an equation, to be treated as block content.
export class MathBlock {
  type = "MathBlock";

  // The identifier for this item
  id?: string;

  // The language used for the equation e.g tex, mathml, asciimath.
  mathLanguage: string;

  // The code of the equation in the `mathLanguage`.
  code: string;

  // A digest of the `code` and `mathLanguage` used to avoid unnecessary transpilation to MathML
  compileDigest?: ExecutionDigest;

  // Errors that occurred when parsing the math equation.
  errors?: string[];

  // The MathML transpiled from the `code`
  mathml?: string;

  // A short label for the math block.
  label?: string;

  constructor(mathLanguage: string, code: string, options?: MathBlock) {
    if (options) Object.assign(this, options)
    this.mathLanguage = mathLanguage;
    this.code = code;
  }
}
