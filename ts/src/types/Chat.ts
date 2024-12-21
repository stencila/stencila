// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { CreativeWork } from "./CreativeWork.js";
import { ModelParameters } from "./ModelParameters.js";

/**
 * A chat conversation, usually with a generative AI model.
 */
export class Chat extends CreativeWork {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Chat";

  /**
   * Model selection and inference parameters.
   */
  modelParameters: ModelParameters;

  /**
   * The id of the system prompt to prefix chat messages with.
   */
  prompt?: string;

  /**
   * The messages, and optionally other content, that make up the conversation.
   */
  content: Block[];

  /**
   * Whether a chat that is nested within another node is ephemeral or not.
   */
  isEphemeral?: boolean;

  constructor(modelParameters: ModelParameters, content: Block[], options?: Partial<Chat>) {
    super();
    this.type = "Chat";
    if (options) Object.assign(this, options);
    this.modelParameters = modelParameters;
    this.content = content;
  }
}

/**
* Create a new `Chat`
*/
export function chat(modelParameters: ModelParameters, content: Block[], options?: Partial<Chat>): Chat {
  return new Chat(modelParameters, content, options);
}
