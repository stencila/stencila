// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Executable } from "./Executable.js";

/**
 * Abstract base type for a document editing instruction.
 */
export class Instruction extends Executable {
  type = "Instruction";

  /**
   * The text of the instruction.
   */
  text: string;

  /**
   * An identifier for the agent assigned to perform the instruction
   */
  assignee?: string;

  constructor(text: string, options?: Partial<Instruction>) {
    super();
    if (options) Object.assign(this, options);
    this.text = text;
  }
}

/**
* Create a new `Instruction`
*/
export function instruction(text: string, options?: Partial<Instruction>): Instruction {
  return new Instruction(text, options);
}
