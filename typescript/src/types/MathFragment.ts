// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Math } from "./Math.js";

/**
 * A fragment of math, e.g a variable name, to be treated as inline content.
 */
export class MathFragment extends Math {
  type = "MathFragment";

  constructor(mathLanguage: string, code: string, options?: Partial<MathFragment>) {
    super(mathLanguage, code);
    if (options) Object.assign(this, options);
    this.mathLanguage = mathLanguage;
    this.code = code;
  }
}

/**
* Create a new `MathFragment`
*/
export function mathFragment(mathLanguage: string, code: string, options?: Partial<MathFragment>): MathFragment {
  return new MathFragment(mathLanguage, code, options);
}
