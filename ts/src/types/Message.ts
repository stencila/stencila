// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from "./Block.js";
import { Entity } from "./Entity.js";
import { MessagePart } from "./MessagePart.js";
import { PersonOrOrganizationOrSoftwareApplication } from "./PersonOrOrganizationOrSoftwareApplication.js";

/**
 * A message from a sender to one or more people, organizations or software application.
 */
export class Message extends Entity {
  type = "Message";

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

  constructor(parts: MessagePart[], options?: Partial<Message>) {
    super();
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
