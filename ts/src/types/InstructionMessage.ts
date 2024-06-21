// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Author } from "./Author.js";
import { Entity } from "./Entity.js";
import { MessagePart } from "./MessagePart.js";
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
   * Parts of the message.
   */
  parts: MessagePart[];

  /**
   * The authors of the message.
   */
  authors?: Author[];

  /**
   * A summary of the provenance of the messages and content within the instruction.
   */
  provenance?: ProvenanceCount[];

  constructor(parts: MessagePart[], options?: Partial<InstructionMessage>) {
    super();
    this.type = "InstructionMessage";
    if (options) Object.assign(this, options);
    this.parts = parts;
  }
}

/**
* Create a new `InstructionMessage`
*/
export function instructionMessage(parts: MessagePart[], options?: Partial<InstructionMessage>): InstructionMessage {
  return new InstructionMessage(parts, options);
}
