// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Inline } from "./Inline.js";
import { Instruction } from "./Instruction.js";
import { Message } from "./Message.js";
import { SuggestionInlineType } from "./SuggestionInlineType.js";

/**
 * An instruction to edit some inline content.
 */
export class InstructionInline extends Instruction {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "InstructionInline";

  /**
   * The content to which the instruction applies.
   */
  content?: Inline[];

  /**
   * A suggestion for the instruction
   */
  suggestion?: SuggestionInlineType;

  constructor(messages: Message[], options?: Partial<InstructionInline>) {
    super(messages);
    this.type = "InstructionInline";
    if (options) Object.assign(this, options);
    this.messages = messages;
  }
}

/**
* Create a new `InstructionInline`
*/
export function instructionInline(messages: Message[], options?: Partial<InstructionInline>): InstructionInline {
  return new InstructionInline(messages, options);
}
