// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Author } from "./Author.js";
import { CompilationDigest } from "./CompilationDigest.js";
import { CompilationError } from "./CompilationError.js";
import { Cord } from "./Cord.js";
import { Entity } from "./Entity.js";

/**
 * Abstract base type for a mathematical variable or equation.
 */
export class Math extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Math";

  /**
   * The code of the equation in the `mathLanguage`.
   */
  code: Cord;

  /**
   * The language used for the equation e.g tex, mathml, asciimath.
   */
  mathLanguage?: string;

  /**
   * The authors of the math.
   */
  authors?: Author[];

  /**
   * A digest of the `code` and `mathLanguage`.
   */
  compilationDigest?: CompilationDigest;

  /**
   * Errors generated when parsing and compiling the math expression.
   */
  compilationErrors?: CompilationError[];

  /**
   * The MathML transpiled from the `code`.
   */
  mathml?: string;

  constructor(code: Cord, options?: Partial<Math>) {
    super();
    this.type = "Math";
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
