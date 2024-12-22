// Generated file; do not edit. See https://github.com/stencila/stencila/tree/main/rust/schema-gen

import { CompilationDigest } from "./CompilationDigest.js";
import { CompilationMessage } from "./CompilationMessage.js";
import { Duration } from "./Duration.js";
import { Entity } from "./Entity.js";
import { ExecutionDependant } from "./ExecutionDependant.js";
import { ExecutionDependency } from "./ExecutionDependency.js";
import { ExecutionKind } from "./ExecutionKind.js";
import { ExecutionMessage } from "./ExecutionMessage.js";
import { ExecutionMode } from "./ExecutionMode.js";
import { ExecutionRequired } from "./ExecutionRequired.js";
import { ExecutionStatus } from "./ExecutionStatus.js";
import { ExecutionTag } from "./ExecutionTag.js";
import { Integer } from "./Integer.js";
import { Timestamp } from "./Timestamp.js";

/**
 * Abstract base type for executable nodes (e.g. `CodeChunk`, `CodeExpression`, `Call`).
 */
export class Executable extends Entity {
  // @ts-expect-error 'not assignable to the same property in base type'
  type: "Executable";

  /**
   * Under which circumstances the node should be executed.
   */
  executionMode?: ExecutionMode;

  /**
   * Under which circumstances child nodes should be executed.
   */
  executionRecursion?: ExecutionMode;

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
   * Status of the most recent, including any current, execution.
   */
  executionStatus?: ExecutionStatus;

  /**
   * The id of the kernel instance that performed the last execution.
   */
  executionInstance?: string;

  /**
   * The kind (e.g. main kernel vs kernel fork) of the last execution.
   */
  executionKind?: ExecutionKind;

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

  constructor(options?: Partial<Executable>) {
    super();
    this.type = "Executable";
    if (options) Object.assign(this, options);
    
  }
}

/**
* Create a new `Executable`
*/
export function executable(options?: Partial<Executable>): Executable {
  return new Executable(options);
}
