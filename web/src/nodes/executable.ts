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
} from "@stencila/types";
import { property } from "lit/decorators.js";

import { Entity } from "./entity";

export class Executable extends Entity {
  @property()
  autoExec?: AutomaticExecution;

  @property()
  compilationDigest?: CompilationDigest;

  @property()
  compilationErrors?: string;

  @property()
  executionDigest?: CompilationDigest;

  @property()
  executionDependencies?: ExecutionDependency;

  @property()
  executionDependants?: ExecutionDependant;

  @property()
  executionTags?: ExecutionTag;

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
}
