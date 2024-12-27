// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Author } from "./Author.js";
import { Block } from "./Block.js";
import { Executable } from "./Executable.js";
import { File } from "./File.js";
import { MessageRole } from "./MessageRole.js";

/**
 * A message within a `Chat`.
 */
export class ChatMessage extends Executable {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "ChatMessage";

  /**
   * The author of the message
   */
  author?: Author;

  /**
   * The role of the message in the conversation.
   */
  role: MessageRole;

  /**
   * The content of the message.
   */
  content: Block[];

  /**
   * The content of the message.
   */
  files?: File[];

  constructor(role: MessageRole, content: Block[], options?: Partial<ChatMessage>) {
    super();
    this.type = "ChatMessage";
    if (options) Object.assign(this, options);
    this.role = role;
    this.content = content;
  }
}

/**
* Create a new `ChatMessage`
*/
export function chatMessage(role: MessageRole, content: Block[], options?: Partial<ChatMessage>): ChatMessage {
  return new ChatMessage(role, content, options);
}
