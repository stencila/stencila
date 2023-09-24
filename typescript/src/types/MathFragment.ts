// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Math } from './Math';

// A fragment of math, e.g a variable name, to be treated as inline content.
export class MathFragment extends Math {
  type = "MathFragment";

  constructor(mathLanguage: string, code: string, options?: MathFragment) {
    super(mathLanguage, code)
    if (options) Object.assign(this, options)
    this.mathLanguage = mathLanguage;
    this.code = code;
  }

  static from(other: MathFragment): MathFragment {
    return new MathFragment(other.mathLanguage!, other.code!, other)
  }
}
