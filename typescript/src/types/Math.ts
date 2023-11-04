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
   * The code of the equation in the `mathLanguage`.
   */
  code: Cord;

  /**
   * The language used for the equation e.g tex, mathml, asciimath.
   */
  mathLanguage?: string;

  /**
   * A digest of the `code` and `mathLanguage`.
   */
  compilationDigest?: ExecutionDigest;

  /**
   * Errors that occurred when parsing and compiling the math equation.
   */
  compilationErrors?: string[];

  /**
   * The MathML transpiled from the `code`.
   */
  mathml?: string;

  constructor(code: Cord, options?: Partial<Math>) {
    super();
    if (options) Object.assign(this, options);
    this.code = code;
  }
}

/**
* Create a new `Math`
*/
export function math(code: Cord, options?: Partial<Math>): Math {
  return new Math(code, options);
}
