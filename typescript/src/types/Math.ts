// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Cord } from "./Cord.js";
import { Entity } from "./Entity.js";
import { ExecutionDigest } from "./ExecutionDigest.js";

/**
 * Abstract base type for a mathematical variable or equation.
 */
export class Math extends Entity {
  type = "Math";

  /**
   * The language used for the equation e.g tex, mathml, asciimath.
   */
  mathLanguage: string;

  /**
   * The code of the equation in the `mathLanguage`.
   */
  code: Cord;

  /**
   * A digest of the `code` and `mathLanguage`.
   */
  compileDigest?: ExecutionDigest;

  /**
   * Errors that occurred when parsing the math equation.
   */
  errors?: string[];

  /**
   * The MathML transpiled from the `code`.
   */
  mathml?: string;

  constructor(mathLanguage: string, code: Cord, options?: Partial<Math>) {
    super();
    if (options) Object.assign(this, options);
    this.mathLanguage = mathLanguage;
    this.code = code;
  }
}

/**
* Create a new `Math`
*/
export function math(mathLanguage: string, code: Cord, options?: Partial<Math>): Math {
  return new Math(mathLanguage, code, options);
}
