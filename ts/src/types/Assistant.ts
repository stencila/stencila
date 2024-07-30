// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { Cord } from "./Cord.js";
import { CreativeWork } from "./CreativeWork.js";
import { InstructionType } from "./InstructionType.js";
import { StringOrNumber } from "./StringOrNumber.js";

/**
 * An assistant for creating and editing document content.
 */
export class Assistant extends CreativeWork {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Assistant";

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
   * The types of instructions that the assistant supports
   */
  instructionTypes: InstructionType[];

  /**
   * The types of nodes that the assistant supports
   */
  nodeTypes: string[];

  /**
   * The content of the assistant's prompt template.
   */
  content: Block[];

  constructor(description: Cord, name: string, version: StringOrNumber, instructionTypes: InstructionType[], nodeTypes: string[], content: Block[], options?: Partial<Assistant>) {
    super();
    this.type = "Assistant";
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
* Create a new `Assistant`
*/
export function assistant(description: Cord, name: string, version: StringOrNumber, instructionTypes: InstructionType[], nodeTypes: string[], content: Block[], options?: Partial<Assistant>): Assistant {
  return new Assistant(description, name, version, instructionTypes, nodeTypes, content, options);
}
