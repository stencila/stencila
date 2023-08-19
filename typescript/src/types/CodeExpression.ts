// Generated file; do not edit. See `../rust/schema-gen` crate.

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
import { Node } from './Node';
import { Timestamp } from './Timestamp';

// An executable programming code expression.
export class CodeExpression {
  type = "CodeExpression";

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

  // The value of the expression when it was last evaluated.
  output?: Node;

  constructor(code: string, programmingLanguage: string, options?: CodeExpression) {
    if (options) Object.assign(this, options)
    this.code = code;
    this.programmingLanguage = programmingLanguage;
  }
}
