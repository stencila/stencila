// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { ChatMessage } from "./ChatMessage.js";
import { Entity } from "./Entity.js";

/**
 * A group of messages, usually alternative model messages, within a `Chat`.
 */
export class ChatMessageGroup extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ChatMessageGroup";

  /**
   * The messages within the group.
   */
  messages: ChatMessage[];

  constructor(messages: ChatMessage[], options?: Partial<ChatMessageGroup>) {
    super();
    this.type = "ChatMessageGroup";
    if (options) Object.assign(this, options);
    this.messages = messages;
  }
}

/**
* Create a new `ChatMessageGroup`
*/
export function chatMessageGroup(messages: ChatMessage[], options?: Partial<ChatMessageGroup>): ChatMessageGroup {
  return new ChatMessageGroup(messages, options);
}
