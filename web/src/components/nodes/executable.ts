import type {
  AutomaticExecution,
  ExecutionDigest,
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
  compilationDigest?: ExecutionDigest;

  @property()
  compilationErrors?: string;

  @property()
  executionDigest?: ExecutionDigest;

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
