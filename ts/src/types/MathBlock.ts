// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Cord } from "./Cord.js";
import { Math } from "./Math.js";

/**
 * A block of math, e.g an equation, to be treated as block content.
 */
export class MathBlock extends Math {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "MathBlock";

  /**
   * A short label for the math block.
   */
  label?: string;

  /**
   * Whether the label should be automatically updated.
   */
  labelAutomatically?: boolean;

  constructor(code: Cord, options?: Partial<MathBlock>) {
    super(code);
    this.type = "MathBlock";
    if (options) Object.assign(this, options);
    this.code = code;
  }
}

/**
* Create a new `MathBlock`
*/
export function mathBlock(code: Cord, options?: Partial<MathBlock>): MathBlock {
  return new MathBlock(code, options);
}
