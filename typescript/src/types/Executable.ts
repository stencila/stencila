// Generated file; do not edit. See `../rust/schema-gen` crate.

import { CodeError } from "./CodeError.js";
import { Duration } from "./Duration.js";
import { Entity } from "./Entity.js";
import { ExecutionAuto } from "./ExecutionAuto.js";
import { ExecutionDependant } from "./ExecutionDependant.js";
import { ExecutionDependency } from "./ExecutionDependency.js";
import { ExecutionDigest } from "./ExecutionDigest.js";
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
  executionAuto?: ExecutionAuto;

  /**
   * A digest of the content, semantics and dependencies of the node.
   */
  compilationDigest?: ExecutionDigest;

  /**
   * The `compileDigest` of the node when it was last executed.
   */
  executionDigest?: ExecutionDigest;

  /**
   * The upstream dependencies of this node.
   */
  executionDependencies?: ExecutionDependency[];

  /**
   * The downstream dependants of this node.
   */
  executionDependants?: ExecutionDependant[];

  /**
   * Tags in the code which affect its execution
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
   * Errors when compiling (e.g. syntax errors) or executing the node.
   */
  errors?: CodeError[];

  constructor(options?: Partial<Executable>) {
    super();
    if (options) Object.assign(this, options);
    
  }

  /**
  * Create a `Executable` from an object
  */
  static from(other: Executable): Executable {
    return new Executable(other);
  }
}
