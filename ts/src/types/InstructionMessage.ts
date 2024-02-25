// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { Entity } from "./Entity.js";
import { MessageLevel } from "./MessageLevel.js";
import { MessagePart } from "./MessagePart.js";
import { PersonOrOrganizationOrSoftwareApplication } from "./PersonOrOrganizationOrSoftwareApplication.js";

/**
 * A message within an `Instruction`.
 */
export class InstructionMessage extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "InstructionMessage";

  /**
   * Parts of the message.
   */
  parts: MessagePart[];

  /**
   * Content of the message.
   */
  content?: Block[];

  /**
   * The authors of the message.
   */
  authors?: PersonOrOrganizationOrSoftwareApplication[];

  /**
   * The severity level of the message.
   */
  level?: MessageLevel;

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
