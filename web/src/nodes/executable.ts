import {
  AutomaticExecution,
  ExecutionRequired,
  ExecutionStatus,
  ExecutionTag,
} from '@stencila/types'
import { apply } from '@twind/core'
import { html } from 'lit'
import { property } from 'lit/decorators.js'
import './exectution-message'

import { Entity } from './entity'

/**
 * Abstract base class for web components representing Stencila Schema `Executable` node types
 *
 * @see https://github.com/stencila/stencila/blob/main/docs/reference/schema/flow/executable.md
 */
export abstract class Executable extends Entity {
  @property({ attribute: 'auto-exec' })
  autoExec?: AutomaticExecution

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

  /**
   * In dynamic view, the executable code can be read and run, but not changed.
   * So display programming language read only and provide buttons for actions
   */
  protected renderExecutableButtons() {
    const containerClasses = apply(['flex flex-row', 'text-base text-black'])
    const dividerClasses = apply(['h-4 w-0', 'border border-black', 'mx-2'])
    return html`
      <div class=${containerClasses}>
        <sl-icon name="deps-tree" library="stencila"></sl-icon>
        <div class=${dividerClasses}></div>
        <sl-icon name="skip-end"></sl-icon>
        <div class=${dividerClasses}></div>
        <sl-icon name="play"></sl-icon>
      </div>
    `
  }

  protected renderActionButtons() {
    return html`<div>TODO: action buttons</div>`
  }
}
