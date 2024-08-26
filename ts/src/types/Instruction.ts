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
   * The type of instruction describing the operation to be performed.
   */
  instructionType: InstructionType;

  /**
   * The instruction message, possibly including images, audio, or other media.
   */
  message?: InstructionMessage;

  /**
   * An identifier for the prompt to be used for the instruction
   */
  prompt?: string;

  /**
   * The name, and other options, for the model that the assistant should use to generate suggestions.
   */
  model?: InstructionModel;

  /**
   * The number of suggestions to generate for the instruction
   */
  replicates?: UnsignedInteger;

  constructor(instructionType: InstructionType, options?: Partial<Instruction>) {
    super();
    this.type = "Instruction";
    if (options) Object.assign(this, options);
    this.instructionType = instructionType;
  }
}

/**
* Create a new `Instruction`
*/
export function instruction(instructionType: InstructionType, options?: Partial<Instruction>): Instruction {
  return new Instruction(instructionType, options);
}
