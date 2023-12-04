import type {
  AutomaticExecution,
  CompilationDigest,
  ExecutionDependency,
  ExecutionDependant,
  ExecutionTag,
  ExecutionRequired,
  Duration,
  ExecutionError,
  Timestamp,
  ExecutionStatus,
  CompilationError,
} from "@stencila/types";
import { html } from "lit";
import { property } from "lit/decorators.js";

import { Entity } from "./entity";

/**
 * Abstract base class for Web Components representing executable nodes
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md
 */
export abstract class Executable extends Entity {
  @property()
  autoExec?: AutomaticExecution;

  @property()
  compilationDigest?: CompilationDigest;

  @property({ type: Array })
  compilationErrors?: CompilationError[];

  @property()
  executionDigest?: CompilationDigest;

  @property({ type: Array })
  executionDependencies?: ExecutionDependency[];

  @property({ type: Array })
  executionDependants?: ExecutionDependant[];

  @property({ type: Array })
  executionTags?: ExecutionTag[];

  @property({ type: Number })
  executionCount?: number;

  @property()
  executionRequired?: ExecutionRequired;

  @property()
  executionKernel?: string;

  @property()
  executionStatus?: ExecutionStatus;

  @property()
  executionEnded?: Timestamp;

  @property()
  executionDuration?: Duration;

  @property()
  executionErrors?: ExecutionError;

  /**
   * Render the `compilationErrors` and `executionErrors` of the node
   *
   * For use by derived custom elements to provide a consistent presentation of
   * errors for a node.
   *
   * TODO: Implement this and other methods for rendering properties of executable nodes
   * https://github.com/stencila/stencila/issues/1786
   */
  protected renderErrors() {
    return html`<div part="errors"></div>`;
  }
}
