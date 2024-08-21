// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { Cord } from "./Cord.js";
import { CreativeWork } from "./CreativeWork.js";
import { InstructionType } from "./InstructionType.js";
import { StringOrNumber } from "./StringOrNumber.js";

/**
 * A prompt for creating or editing document content.
 */
export class Prompt extends CreativeWork {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Prompt";

  /**
   * A description of the item.
   */
  description: Cord;

  /**
   * The name of the item.
   */
  name: string;

  /**
   * The version of the creative work.
   */
  version: StringOrNumber;

  /**
   * The types of instructions that the prompt supports
   */
  instructionTypes: InstructionType[];

  /**
   * Regular expressions used to match the prompt with a user instruction
   */
  instructionPatterns?: string[];

  /**
   * The types of nodes that the prompt supports
   */
  nodeTypes: string[];

  /**
   * The content of the prompt.
   */
  content: Block[];

  constructor(description: Cord, name: string, version: StringOrNumber, instructionTypes: InstructionType[], nodeTypes: string[], content: Block[], options?: Partial<Prompt>) {
    super();
    this.type = "Prompt";
    if (options) Object.assign(this, options);
    this.description = description;
    this.name = name;
    this.version = version;
    this.instructionTypes = instructionTypes;
    this.nodeTypes = nodeTypes;
    this.content = content;
  }
}

/**
* Create a new `Prompt`
*/
export function prompt(description: Cord, name: string, version: StringOrNumber, instructionTypes: InstructionType[], nodeTypes: string[], content: Block[], options?: Partial<Prompt>): Prompt {
  return new Prompt(description, name, version, instructionTypes, nodeTypes, content, options);
}
