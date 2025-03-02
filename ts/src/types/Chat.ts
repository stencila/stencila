// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { CreativeWork } from "./CreativeWork.js";
import { ModelParameters } from "./ModelParameters.js";
import { PromptBlock } from "./PromptBlock.js";
import { SuggestionBlock } from "./SuggestionBlock.js";

/**
 * A chat conversation, usually with a generative AI model.
 */
export class Chat extends CreativeWork {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Chat";

  /**
   * Whether the chat is embedded within a document (i.e. is not standalone).
   */
  isEmbedded?: boolean;

  /**
   * The prompt selected, rendered and provided to the model
   */
  prompt: PromptBlock;

  /**
   * Model selection and inference parameters.
   */
  modelParameters: ModelParameters;

  /**
   * The messages, and optionally other content, that make up the chat.
   */
  content: Block[];

  /**
   * Suggestions of content that is the focus of the chat.
   */
  suggestions?: SuggestionBlock[];

  constructor(prompt: PromptBlock, modelParameters: ModelParameters, content: Block[], options?: Partial<Chat>) {
    super();
    this.type = "Chat";
    if (options) Object.assign(this, options);
    this.prompt = prompt;
    this.modelParameters = modelParameters;
    this.content = content;
  }
}

/**
* Create a new `Chat`
*/
export function chat(prompt: PromptBlock, modelParameters: ModelParameters, content: Block[], options?: Partial<Chat>): Chat {
  return new Chat(prompt, modelParameters, content, options);
}
