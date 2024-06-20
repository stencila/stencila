// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Executable } from "./Executable.js";
import { InstructionMessage } from "./InstructionMessage.js";
import { InstructionModel } from "./InstructionModel.js";
import { InstructionType } from "./InstructionType.js";
import { UnsignedInteger } from "./UnsignedInteger.js";

/**
 * Abstract base type for a document editing instruction.
 */
export class Instruction extends Executable {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Instruction";

  /**
   * The type of instruction.
   */
  instructionType: InstructionType;

  /**
   * Messages involved in the instruction.
   */
  messages: InstructionMessage[];

  /**
   * An identifier for the assistant assigned to perform the instruction
   */
  assignee?: string;

  /**
   * The name, and other options, for the model that the assistant should use to generate suggestions.
   */
  model?: InstructionModel;

  /**
   * The number of suggestions to generate for the instruction
   */
  replicates?: UnsignedInteger;

  /**
   * Whether suggestions should be hidden in source views such as Markdown.
   */
  hideSuggestions?: boolean;

  constructor(instructionType: InstructionType, messages: InstructionMessage[], options?: Partial<Instruction>) {
    super();
    this.type = "Instruction";
    if (options) Object.assign(this, options);
    this.instructionType = instructionType;
    this.messages = messages;
  }
}

/**
* Create a new `Instruction`
*/
export function instruction(instructionType: InstructionType, messages: InstructionMessage[], options?: Partial<Instruction>): Instruction {
  return new Instruction(instructionType, messages, options);
}
