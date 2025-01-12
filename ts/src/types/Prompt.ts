// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { CreativeWork } from "./CreativeWork.js";
import { Inline } from "./Inline.js";
import { InstructionType } from "./InstructionType.js";
import { StringOrNumber } from "./StringOrNumber.js";
import { UnsignedIntegerOrString } from "./UnsignedIntegerOrString.js";

/**
 * A prompt for creating or editing document content.
 */
export class Prompt extends CreativeWork {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Prompt";

  /**
   * A description of the item.
   */
  description: string;

  /**
   * The name of the item.
   */
  name: string;

  /**
   * The title of the creative work.
   */
  title: Inline[];

  /**
   * The version of the creative work.
   */
  version: StringOrNumber;

  /**
   * The types of instructions that the prompt supports
   */
  instructionTypes: InstructionType[];

  /**
   * The types of nodes that the prompt supports
   */
  nodeTypes?: string[];

  /**
   * The number of nodes that the prompt supports
   */
  nodeCount?: UnsignedIntegerOrString;

  /**
   * Regular expressions used to match the prompt with a user query
   */
  queryPatterns?: string[];

  /**
   * The content of the prompt.
   */
  content: Block[];

  constructor(description: string, name: string, title: Inline[], version: StringOrNumber, instructionTypes: InstructionType[], content: Block[], options?: Partial<Prompt>) {
    super();
    this.type = "Prompt";
    if (options) Object.assign(this, options);
    this.description = description;
    this.name = name;
    this.title = title;
    this.version = version;
    this.instructionTypes = instructionTypes;
    this.content = content;
  }
}

/**
* Create a new `Prompt`
*/
export function prompt(description: string, name: string, title: Inline[], version: StringOrNumber, instructionTypes: InstructionType[], content: Block[], options?: Partial<Prompt>): Prompt {
  return new Prompt(description, name, title, version, instructionTypes, content, options);
}
