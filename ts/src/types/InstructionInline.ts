// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Inline } from "./Inline.js";
import { Instruction } from "./Instruction.js";

/**
 * An instruction to edit some inline content.
 */
export class InstructionInline extends Instruction {
  type = "InstructionInline";

  /**
   * The content to which the instruction applies.
   */
  content?: Inline[];

  constructor(text: string, options?: Partial<InstructionInline>) {
    super(text);
    if (options) Object.assign(this, options);
    this.text = text;
  }
}

/**
* Create a new `InstructionInline`
*/
export function instructionInline(text: string, options?: Partial<InstructionInline>): InstructionInline {
  return new InstructionInline(text, options);
}
