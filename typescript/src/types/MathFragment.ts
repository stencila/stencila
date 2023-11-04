// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Cord } from "./Cord.js";
import { Math } from "./Math.js";

/**
 * A fragment of math, e.g a variable name, to be treated as inline content.
 */
export class MathFragment extends Math {
  type = "MathFragment";

  constructor(code: Cord, options?: Partial<MathFragment>) {
    super(code);
    if (options) Object.assign(this, options);
    this.code = code;
  }
}

/**
* Create a new `MathFragment`
*/
export function mathFragment(code: Cord, options?: Partial<MathFragment>): MathFragment {
  return new MathFragment(code, options);
}
