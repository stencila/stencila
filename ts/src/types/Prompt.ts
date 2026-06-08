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
import { Inline } from "./Inline.js";
import { InstructionType } from "./InstructionType.js";
import { Integer } from "./Integer.js";
import { StringOrNumber } from "./StringOrNumber.js";
import { Timestamp } from "./Timestamp.js";
import { UnsignedIntegerOrString } from "./UnsignedIntegerOrString.js";

/**
 * A prompt for creating or editing document content.
 */
export class Prompt extends CreativeWork {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Prompt";

  /**
   * A description of the item.
   */
  description: string;

  /**
   * The name of the item.
   */
  name: string;

  /**
   * The title of the creative work.
   */
  title: Inline[];

  /**
   * The version of the creative work.
   */
  version: StringOrNumber;

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
   * Frontmatter containing document metadata.
   */
  frontmatter?: string;

  /**
   * The types of instructions that the prompt supports
   */
  instructionTypes: InstructionType[];

  /**
   * The types of nodes that the prompt supports
   */
  nodeTypes?: string[];

  /**
   * The number of nodes that the prompt supports
   */
  nodeCount?: UnsignedIntegerOrString;

  /**
   * Regular expressions used to match the prompt with a user query
   */
  queryPatterns?: string[];

  /**
   * The content of the prompt.
   */
  content: Block[];

  constructor(description: string, name: string, title: Inline[], version: StringOrNumber, instructionTypes: InstructionType[], content: Block[], options?: Partial<Prompt>) {
    super();
    this.type = "Prompt";
    if (options) Object.assign(this, options);
    this.description = description;
    this.name = name;
    this.title = title;
    this.version = version;
    this.instructionTypes = instructionTypes;
    this.content = content;
  }
}

/**
* Create a new `Prompt`
*/
export function prompt(description: string, name: string, title: Inline[], version: StringOrNumber, instructionTypes: InstructionType[], content: Block[], options?: Partial<Prompt>): Prompt {
  return new Prompt(description, name, title, version, instructionTypes, content, options);
}
