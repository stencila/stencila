// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from "./Inline.js";
import { Instruct } from "./Instruct.js";

/**
 * An instruction to edit some inline content.
 */
export class InstructInline extends Instruct {
  type = "InstructInline";

  /**
   * The content to which the instruction applies.
   */
  content?: Inline[];

  constructor(text: string, options?: Partial<InstructInline>) {
    super(text);
    if (options) Object.assign(this, options);
    this.text = text;
  }
}

/**
* Create a new `InstructInline`
*/
export function instructInline(text: string, options?: Partial<InstructInline>): InstructInline {
  return new InstructInline(text, options);
}
