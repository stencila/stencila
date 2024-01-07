// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from "./Block.js";
import { Instruction } from "./Instruction.js";
import { Message } from "./Message.js";
import { SuggestionBlockType } from "./SuggestionBlockType.js";

/**
 * An instruction to edit some block content.
 */
export class InstructionBlock extends Instruction {
  type = "InstructionBlock";

  /**
   * The content to which the instruction applies.
   */
  content?: Block[];

  /**
   * A suggestion for the instruction
   */
  suggestion?: SuggestionBlockType;

  constructor(messages: Message[], options?: Partial<InstructionBlock>) {
    super(messages);
    if (options) Object.assign(this, options);
    this.messages = messages;
  }
}

/**
* Create a new `InstructionBlock`
*/
export function instructionBlock(messages: Message[], options?: Partial<InstructionBlock>): InstructionBlock {
  return new InstructionBlock(messages, options);
}
