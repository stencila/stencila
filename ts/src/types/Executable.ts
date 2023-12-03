// Generated file; do not edit. See `../rust/schema-gen` crate.

import { AutomaticExecution } from "./AutomaticExecution.js";
import { CompilationDigest } from "./CompilationDigest.js";
import { CompilationError } from "./CompilationError.js";
import { Duration } from "./Duration.js";
import { Entity } from "./Entity.js";
import { ExecutionDependant } from "./ExecutionDependant.js";
import { ExecutionDependency } from "./ExecutionDependency.js";
import { ExecutionError } from "./ExecutionError.js";
import { ExecutionRequired } from "./ExecutionRequired.js";
import { ExecutionStatus } from "./ExecutionStatus.js";
import { ExecutionTag } from "./ExecutionTag.js";
import { Integer } from "./Integer.js";
import { Timestamp } from "./Timestamp.js";

/**
 * Abstract base type for executable nodes (e.g. `CodeChunk`, `CodeExpression`, `Call`).
 */
export class Executable extends Entity {
  type = "Executable";

  /**
   * Under which circumstances the code should be automatically executed.
   */
  autoExec?: AutomaticExecution;

  /**
   * A digest of the content, semantics and dependencies of the node.
   */
  compilationDigest?: CompilationDigest;

  /**
   * Errors generated when compiling the code.
   */
  compilationErrors?: CompilationError[];

  /**
   * The `compilationDigest` of the node when it was last executed.
   */
  executionDigest?: CompilationDigest;

  /**
   * The upstream dependencies of this node.
   */
  executionDependencies?: ExecutionDependency[];

  /**
   * The downstream dependants of this node.
   */
  executionDependants?: ExecutionDependant[];

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
   * The id of the kernel that the node was last executed in.
   */
  executionKernel?: string;

  /**
   * Status of the most recent, including any current, execution.
   */
  executionStatus?: ExecutionStatus;

  /**
   * The timestamp when the last execution ended.
   */
  executionEnded?: Timestamp;

  /**
   * Duration of the last execution.
   */
  executionDuration?: Duration;

  /**
   * Errors when executing the node.
   */
  executionErrors?: ExecutionError[];

  constructor(options?: Partial<Executable>) {
    super();
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Executable`
*/
export function executable(options?: Partial<Executable>): Executable {
  return new Executable(options);
}
