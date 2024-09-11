import {
  ExecutionMode,
  ExecutionRequired,
  ExecutionStatus,
  ExecutionTag,
} from '@stencila/types'
import { property } from 'lit/decorators.js'

import { Entity } from './entity'

/**
 * Abstract base class for web components representing Stencila Schema `Executable` node types
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md
 */
export abstract class Executable extends Entity {
  @property({ attribute: 'execution-mode' })
  executionMode?: ExecutionMode

  @property({ attribute: 'execution-tags', type: Array })
  executionTags?: ExecutionTag[]

  @property({ attribute: 'execution-count', type: Number })
  executionCount?: number

  @property({ attribute: 'execution-required' })
  executionRequired?: ExecutionRequired

  @property({ attribute: 'execution-status' })
  executionStatus?: ExecutionStatus

  @property({ attribute: 'execution-ended', type: Number })
  executionEnded?: number

  @property({ attribute: 'execution-duration', type: Number })
  executionDuration?: number
}
