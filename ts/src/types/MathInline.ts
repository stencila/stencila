// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Cord } from "./Cord.js";
import { Math } from "./Math.js";

/**
 * A fragment of math, e.g a variable name, to be treated as inline content.
 */
export class MathInline extends Math {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "MathInline";

  constructor(code: Cord, options?: Partial<MathInline>) {
    super(code);
    this.type = "MathInline";
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
