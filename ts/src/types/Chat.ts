// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { Block } from "./Block.js";
import { CompilationDigest } from "./CompilationDigest.js";
import { CompilationMessage } from "./CompilationMessage.js";
import { CreativeWork } from "./CreativeWork.js";
import { Duration } from "./Duration.js";
import { ExecutionMessage } from "./ExecutionMessage.js";
import { ExecutionMode } from "./ExecutionMode.js";
import { ExecutionRequired } from "./ExecutionRequired.js";
import { ExecutionStatus } from "./ExecutionStatus.js";
import { ExecutionTag } from "./ExecutionTag.js";
import { Integer } from "./Integer.js";
import { ModelParameters } from "./ModelParameters.js";
import { PromptBlock } from "./PromptBlock.js";
import { Timestamp } from "./Timestamp.js";

/**
 * A chat conversation, usually with a generative AI model.
 */
export class Chat extends CreativeWork {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Chat";

  /**
   * Under which circumstances the node should be executed.
   */
  executionMode?: ExecutionMode;

  /**
   * A digest of the content, semantics and dependencies of the node.
   */
  compilationDigest?: CompilationDigest;

  /**
   * Messages generated while compiling the code.
   */
  compilationMessages?: CompilationMessage[];

  /**
   * The `compilationDigest` of the node when it was last executed.
   */
  executionDigest?: CompilationDigest;

  /**
   * Tags in the code which affect its execution.
   */
  executionTags?: ExecutionTag[];

  /**
   * A count of the number of times that the node has been executed.
   */
  executionCount?: Integer;

  /**
   * Whether, and why, the code requires execution or re-execution.
   */
  executionRequired?: ExecutionRequired;

  /**
   * Status of the most recent, including any current, execution.
   */
  executionStatus?: ExecutionStatus;

  /**
   * The id of the kernel instance that performed the last execution.
   */
  executionInstance?: string;

  /**
   * The timestamp when the last execution ended.
   */
  executionEnded?: Timestamp;

  /**
   * Duration of the last execution.
   */
  executionDuration?: Duration;

  /**
   * Messages emitted while executing the node.
   */
  executionMessages?: ExecutionMessage[];

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
   * The ids of the nodes that this chat is targeting
   */
  targetNodes?: string[];

  /**
   * The messages, and optionally other content, that make up the chat.
   */
  content: Block[];

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
