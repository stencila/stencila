// Generated file; do not edit. See `../rust/schema-gen` crate.

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
import { String } from './String';
import { Timestamp } from './Timestamp';

// Include content from an external source (e.g. file, URL).
export class Include {
  // The type of this item
  type = "Include";

  // The identifier for this item
  id?: String;

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
  executionKernel?: String;

  // Status of the most recent, including any current, execution.
  executionStatus?: ExecutionStatus;

  // The timestamp when the last execution ended.
  executionEnded?: Timestamp;

  // Duration of the last execution.
  executionDuration?: Duration;

  // Errors when compiling (e.g. syntax errors) or executing the node.
  errors?: CodeError[];

  // The external source of the content, a file path or URL.
  source: String;

  // Media type of the source content.
  mediaType?: String;

  // A query to select a subset of content from the source
  select?: String;

  // The structured content decoded from the source.
  content?: Block[];

  constructor(source: String, options?: Include) {
    if (options) Object.assign(this, options)
    this.source = source;
  }
}
