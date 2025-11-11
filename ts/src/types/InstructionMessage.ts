// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Author } from "./Author.js";
import { Entity } from "./Entity.js";
import { File } from "./File.js";
import { Inline } from "./Inline.js";
import { MessageRole } from "./MessageRole.js";
import { ProvenanceCount } from "./ProvenanceCount.js";

/**
 * A message within an `Instruction`.
 */
export class InstructionMessage extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "InstructionMessage";

  /**
   * The role of the message in the conversation.
   */
  role?: MessageRole;

  /**
   * The content of the message as inline nodes.
   */
  content: Inline[];

  /**
   * Files attached to the message.
   */
  files?: File[];

  /**
   * The authors of the message.
   */
  authors?: Author[];

  /**
   * A summary of the provenance of the messages and content within the instruction.
   */
  provenance?: ProvenanceCount[];

  constructor(content: Inline[], options?: Partial<InstructionMessage>) {
    super();
    this.type = "InstructionMessage";
    if (options) Object.assign(this, options);
    this.content = content;
  }
}

/**
* Create a new `InstructionMessage`
*/
export function instructionMessage(content: Inline[], options?: Partial<InstructionMessage>): InstructionMessage {
  return new InstructionMessage(content, options);
}
