// Generated file; do not edit. See `../rust/schema-gen` crate.

import { Block } from './Block';
import { CallArgument } from './CallArgument';
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

// Call another document, optionally with arguments, and include its executed content.
export class Call {
  type = "Call";

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

  // The external source of the content, a file path or URL.
  source: string;

  // Media type of the source content.
  mediaType?: string;

  // A query to select a subset of content from the source
  select?: string;

  // The structured content decoded from the source.
  content?: Block[];

  // The value of the source document's parameters to call it with
  arguments: CallArgument[];

  constructor(source: string, args: CallArgument[], options?: Call) {
    if (options) Object.assign(this, options)
    this.source = source;
    this.arguments = args;
  }
}
