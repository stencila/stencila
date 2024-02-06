// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { Instruction } from "./Instruction.js";
import { Message } from "./Message.js";
import { SuggestionBlockType } from "./SuggestionBlockType.js";

/**
 * An instruction to edit some block content.
 */
export class InstructionBlock extends Instruction {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "InstructionBlock";

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
    this.type = "InstructionBlock";
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
