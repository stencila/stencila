// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Math } from "./Math.js";

/**
 * A block of math, e.g an equation, to be treated as block content.
 */
export class MathBlock extends Math {
  type = "MathBlock";

  /**
   * A short label for the math block.
   */
  label?: string;

  constructor(mathLanguage: string, code: string, options?: Partial<MathBlock>) {
    super(mathLanguage, code);
    if (options) Object.assign(this, options);
    this.mathLanguage = mathLanguage;
    this.code = code;
  }

  /**
  * Create a `MathBlock` from an object
  */
  static from(other: MathBlock): MathBlock {
    return new MathBlock(other.mathLanguage!, other.code!, other);
  }
}
