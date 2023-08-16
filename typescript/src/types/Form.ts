// Generated file. Do not edit; see `rust/schema-gen` crate.

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
import { FormDeriveAction } from './FormDeriveAction';
import { Integer } from './Integer';
import { IntegerOrString } from './IntegerOrString';
import { String } from './String';
import { Timestamp } from './Timestamp';

// A form to batch updates in document parameters
export class Form {
  // The type of this item
  type = "Form";

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

  // The content within the form, usually containing at least one `Parameter`.
  content: Block[];

  // The dotted path to the object (e.g a database table) that the form should be derived from
  deriveFrom?: String;

  // The action (create, update or delete) to derive for the form
  deriveAction?: FormDeriveAction;

  // An identifier for the item to be the target of Update or Delete actions
  deriveItem?: IntegerOrString;

  constructor(content: Block[], options?: Form) {
    if (options) Object.assign(this, options)
    this.content = content;
  }
}
