// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { Entity } from "./Entity.js";
import { MessageLevel } from "./MessageLevel.js";
import { MessagePart } from "./MessagePart.js";
import { PersonOrOrganizationOrSoftwareApplication } from "./PersonOrOrganizationOrSoftwareApplication.js";

/**
 * A message from a sender to one or more people, organizations or software application.
 */
export class Message extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Message";

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

  constructor(parts: MessagePart[], options?: Partial<Message>) {
    super();
    this.type = "Message";
    if (options) Object.assign(this, options);
    this.parts = parts;
  }
}

/**
* Create a new `Message`
*/
export function message(parts: MessagePart[], options?: Partial<Message>): Message {
  return new Message(parts, options);
}
