import {
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
} from '@stencila/types'
import { html } from 'lit'
import { property } from 'lit/decorators.js'

import { Entity } from './entity'

/**
 * Abstract base class for web components representing Stencila Schema `Executable` node types
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md
 */
export abstract class Executable extends Entity {
  @property({ attribute: 'auto-exec' })
  autoExec?: AutomaticExecution

  @property({ attribute: 'compilation-digest', type: Object })
  compilationDigest?: CompilationDigest

  @property({ attribute: 'compilation-errors', type: Array })
  compilationErrors?: CompilationError[]

  @property({ attribute: 'execution-digest', type: Object })
  executionDigest?: CompilationDigest

  @property({ attribute: 'execution-dependencies', type: Array })
  executionDependencies?: ExecutionDependency[]

  @property({ attribute: 'execution-dependants', type: Array })
  executionDependants?: ExecutionDependant[]

  @property({ attribute: 'execution-tags', type: Array })
  executionTags?: ExecutionTag[]

  @property({ attribute: 'execution-count', type: Number })
  executionCount?: number

  @property({ attribute: 'execution-required' })
  executionRequired?: ExecutionRequired

  @property({ attribute: 'execution-kernel' })
  executionKernel?: string

  @property({ attribute: 'execution-status' })
  executionStatus?: ExecutionStatus

  @property({ attribute: 'execution-ended', type: Object })
  executionEnded?: Timestamp

  @property({ attribute: 'execution-duration', type: Object })
  executionDuration?: Duration

  @property({ attribute: 'execution-errors', type: Array })
  executionErrors?: ExecutionError[]

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
    return html`<div part="errors"></div>`
  }
}
