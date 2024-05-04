// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Author } from "./Author.js";
import { Executable } from "./Executable.js";
import { InstructionMessage } from "./InstructionMessage.js";
import { ProvenanceCount } from "./ProvenanceCount.js";

/**
 * Abstract base type for a document editing instruction.
 */
export class Instruction extends Executable {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Instruction";

  /**
   * Messages involved in the instruction.
   */
  messages: InstructionMessage[];

  /**
   * A list of candidates for the assignee property.
   */
  candidates?: string[];

  /**
   * An identifier for the agent assigned to perform the instruction
   */
  assignee?: string;

  /**
   * The authors of the instruction.
   */
  authors?: Author[];

  /**
   * A summary of the provenance of the messages and content within the instruction.
   */
  provenance?: ProvenanceCount[];

  constructor(messages: InstructionMessage[], options?: Partial<Instruction>) {
    super();
    this.type = "Instruction";
    if (options) Object.assign(this, options);
    this.messages = messages;
  }
}

/**
* Create a new `Instruction`
*/
export function instruction(messages: InstructionMessage[], options?: Partial<Instruction>): Instruction {
  return new Instruction(messages, options);
}
