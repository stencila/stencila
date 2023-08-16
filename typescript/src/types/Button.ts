// Generated file. Do not edit; see `rust/schema-gen` crate.

import { Boolean } from './Boolean';
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

// A button.
export class Button {
  // The type of this item
  type = "Button";

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

  // The code.
  code: String;

  // The programming language of the code.
  programmingLanguage: String;

  // Whether the programming language of the code should be guessed based on syntax and variables used
  guessLanguage?: Boolean;

  // Media type, typically expressed using a MIME format, of the code.
  mediaType?: String;

  // The name of the variable associated with the button.
  name: String;

  // A label for the button
  label?: String;

  // Whether the button is currently disabled
  isDisabled?: Boolean;

  constructor(code: String, programmingLanguage: String, name: String, options?: Button) {
    if (options) Object.assign(this, options)
    this.code = code;
    this.programmingLanguage = programmingLanguage;
    this.name = name;
  }
}
