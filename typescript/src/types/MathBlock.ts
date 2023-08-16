// Generated file. Do not edit; see `rust/schema-gen` crate.

import { ExecutionDigest } from './ExecutionDigest';
import { String } from './String';

// A block of math, e.g an equation, to be treated as block content.
export class MathBlock {
  // The type of this item
  type = "MathBlock";

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

  // A short label for the math block.
  label?: String;

  constructor(mathLanguage: String, code: String, options?: MathBlock) {
    if (options) Object.assign(this, options)
    this.mathLanguage = mathLanguage;
    this.code = code;
  }
}
