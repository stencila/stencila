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
   * The prompt selected, rendered and provided to the model
   */
  prompt: PromptBlock;

  /**
   * The instruction message, possibly including images, audio, or other media.
   */
  message: InstructionMessage;

  /**
   * Model selection and inference parameters.
   */
  modelParameters: ModelParameters;

  /**
   * The index of the suggestion that is currently active
   */
  activeSuggestion?: UnsignedInteger;

  constructor(instructionType: InstructionType, prompt: PromptBlock, message: InstructionMessage, modelParameters: ModelParameters, options?: Partial<Instruction>) {
    super();
    this.type = "Instruction";
    if (options) Object.assign(this, options);
    this.instructionType = instructionType;
    this.prompt = prompt;
    this.message = message;
    this.modelParameters = modelParameters;
  }
}

/**
* Create a new `Instruction`
*/
export function instruction(instructionType: InstructionType, prompt: PromptBlock, message: InstructionMessage, modelParameters: ModelParameters, options?: Partial<Instruction>): Instruction {
  return new Instruction(instructionType, prompt, message, modelParameters, options);
}
