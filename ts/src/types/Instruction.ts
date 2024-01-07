// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Author } from "./Author.js";
import { Executable } from "./Executable.js";
import { Message } from "./Message.js";

/**
 * Abstract base type for a document editing instruction.
 */
export class Instruction extends Executable {
  type = "Instruction";

  /**
   * Messages involved in the instruction.
   */
  messages: Message[];

  /**
   * An identifier for the agent assigned to perform the instruction
   */
  assignee?: string;

  /**
   * The authors of the instruction.
   */
  authors?: Author[];

  constructor(messages: Message[], options?: Partial<Instruction>) {
    super();
    if (options) Object.assign(this, options);
    this.messages = messages;
  }
}

/**
* Create a new `Instruction`
*/
export function instruction(messages: Message[], options?: Partial<Instruction>): Instruction {
  return new Instruction(messages, options);
}
