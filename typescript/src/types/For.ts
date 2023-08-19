// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Array } from './Array';
import { Block } from './Block';
import { CodeError } from './CodeError';
import { Duration } from './Duration';
import { ExecutionAuto } from './ExecutionAuto';
import { ExecutionDependant } from './ExecutionDependant';
import { ExecutionDependency } from './ExecutionDependency';
import { ExecutionDigest } from './ExecutionDigest';
import { ExecutionRequired } from './ExecutionRequired';
import { ExecutionStatus } from './ExecutionStatus';
import { ExecutionTag } from './ExecutionTag';
import { Integer } from './Integer';
import { Timestamp } from './Timestamp';

// Repeat a block content for each item in an array.
export class For {
  type = "For";

  // The identifier for this item
  id?: string;

  // Under which circumstances the code should be automatically executed.
  executionAuto?: ExecutionAuto;

  // A digest of the content, semantics and dependencies of the node.
  compilationDigest?: ExecutionDigest;

  // The `compileDigest` of the node when it was last executed.
  executionDigest?: ExecutionDigest;

  // The upstream dependencies of this node.
  executionDependencies?: ExecutionDependency[];

  // The downstream dependants of this node.
  executionDependants?: ExecutionDependant[];

  // Tags in the code which affect its execution
  executionTags?: ExecutionTag[];

  // A count of the number of times that the node has been executed.
  executionCount?: Integer;

  // Whether, and why, the code requires execution or re-execution.
  executionRequired?: ExecutionRequired;

  // The id of the kernel that the node was last executed in.
  executionKernel?: string;

  // Status of the most recent, including any current, execution.
  executionStatus?: ExecutionStatus;

  // The timestamp when the last execution ended.
  executionEnded?: Timestamp;

  // Duration of the last execution.
  executionDuration?: Duration;

  // Errors when compiling (e.g. syntax errors) or executing the node.
  errors?: CodeError[];

  // The code.
  code: string;

  // The programming language of the code.
  programmingLanguage: string;

  // Whether the programming language of the code should be guessed based on syntax and variables used
  guessLanguage?: boolean;

  // Media type, typically expressed using a MIME format, of the code.
  mediaType?: string;

  // The name to give to the variable representing each item in the iterated array
  symbol: string;

  // The content to repeat for each item
  content: Block[];

  // The content to render if there are no items
  otherwise?: Block[];

  // The content repeated for each iteration
  iterations?: Array[];

  constructor(code: string, programmingLanguage: string, symbol: string, content: Block[], options?: For) {
    if (options) Object.assign(this, options)
    this.code = code;
    this.programmingLanguage = programmingLanguage;
    this.symbol = symbol;
    this.content = content;
  }
}
