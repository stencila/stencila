// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { CreativeWork } from "./CreativeWork.js";
import { InstructionModel } from "./InstructionModel.js";

/**
 * A chat conversation, usually with a generative AI model.
 */
export class Chat extends CreativeWork {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Chat";

  /**
   * The name, and other options, for the model involved in the chat.
   */
  model: InstructionModel;

  /**
   * The id of the system prompt to prefix chat messages with.
   */
  prompt?: string;

  /**
   * The messages, and optionally other content, that make up the conversation.
   */
  content: Block[];

  constructor(model: InstructionModel, content: Block[], options?: Partial<Chat>) {
    super();
    this.type = "Chat";
    if (options) Object.assign(this, options);
    this.model = model;
    this.content = content;
  }
}

/**
* Create a new `Chat`
*/
export function chat(model: InstructionModel, content: Block[], options?: Partial<Chat>): Chat {
  return new Chat(model, content, options);
}
