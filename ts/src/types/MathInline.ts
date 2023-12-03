// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Cord } from "./Cord.js";
import { Math } from "./Math.js";

/**
 * A fragment of math, e.g a variable name, to be treated as inline content.
 */
export class MathInline extends Math {
  type = "MathInline";

  constructor(code: Cord, options?: Partial<MathInline>) {
    super(code);
    if (options) Object.assign(this, options);
    this.code = code;
  }
}

/**
* Create a new `MathInline`
*/
export function mathInline(code: Cord, options?: Partial<MathInline>): MathInline {
  return new MathInline(code, options);
}
