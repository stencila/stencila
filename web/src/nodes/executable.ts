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

export abstract class Executable extends Entity {
  @property()
  autoExec?: AutomaticExecution;

  @property()
  compilationDigest?: CompilationDigest;

  @property()
  compilationErrors?: CompilationError[];

  @property()
  executionDigest?: CompilationDigest;

  @property()
  executionDependencies?: ExecutionDependency[];

  @property()
  executionDependants?: ExecutionDependant[];

  @property()
  executionTags?: ExecutionTag[];

  @property()
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
   */
  renderErrors() {
    return html`
        <div part="error">
          <!-- TODO -->
        </div>
      `
  }
}
