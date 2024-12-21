// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Executable } from "./Executable.js";
import { InstructionMessage } from "./InstructionMessage.js";
import { InstructionType } from "./InstructionType.js";
import { ModelParameters } from "./ModelParameters.js";
import { PromptBlock } from "./PromptBlock.js";
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
   * Model selection and inference parameters.
   */
  modelParameters: ModelParameters;

  /**
   * A string identifying which operations should, or should not, automatically be applied to generated suggestions.
   */
  recursion?: string;

  /**
   * The prompt chosen, rendered and provided to the model
   */
  promptProvided?: PromptBlock;

  /**
   * The index of the suggestion that is currently active
   */
  activeSuggestion?: UnsignedInteger;

  constructor(instructionType: InstructionType, modelParameters: ModelParameters, options?: Partial<Instruction>) {
    super();
    this.type = "Instruction";
    if (options) Object.assign(this, options);
    this.instructionType = instructionType;
    this.modelParameters = modelParameters;
  }
}

/**
* Create a new `Instruction`
*/
export function instruction(instructionType: InstructionType, modelParameters: ModelParameters, options?: Partial<Instruction>): Instruction {
  return new Instruction(instructionType, modelParameters, options);
}
