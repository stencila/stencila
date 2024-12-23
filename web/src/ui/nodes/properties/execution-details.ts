import {
  ExecutionBounds,
  ExecutionMode,
  ExecutionRequired,
  ExecutionStatus,
  ExecutionTag,
} from '@stencila/types'
import { apply } from '@twind/core'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../twind'
import { UIBaseClass } from '../mixins/ui-base-class'

import './execution-bounded'
import './execution-bounds'
import './execution-count'
import './execution-duration'
import './execution-ended'
import './execution-mode'
import './execution-state'

/**
 * A component for displaying various execution related property of executable nodes
 *
 * Acts as a container for execution details which can be collapsed or expanded.
 * Having this collapsable is important because the user may not always want to
 * see details such as all the dependants of a node.
 *
 * TODO: Render `autoExec`, `executionTags`, `executionDependencies`, and `executionDependants`
 * when then are available in documents (they are not yet re-implemented)
 */
@customElement('stencila-ui-node-execution-details')
@withTwind()
export class UINodeExecutionDetails extends UIBaseClass {
  @property()
  mode?: ExecutionMode

  @property()
  bounds?: ExecutionBounds

  @property({ type: Array })
  tags?: ExecutionTag[]

  @property({ type: Number })
  count?: number

  @property()
  required?: ExecutionRequired = 'NeverExecuted'

  @property()
  status?: ExecutionStatus

  @property()
  bounded?: ExecutionBounds

  @property({ type: Number })
  ended?: number

  @property({ type: Number })
  duration?: number

  override render() {
    const { colour, borderColour } = this.ui

    const classes = apply([
      'flex flex-row flex-wrap items-center justify-between gap-3',
      'text-xs leading-tight',
      'min-h-[2.25rem]',
      'py-1.5 px-4',
      `bg-[${colour}]`,
      `border-t border-[${borderColour}]`,
      'font-sans',
    ])

    return html`
      <div class=${classes}>
        ${this.type !== 'SuggestionBlock'
          ? this.renderAllDetails()
          : this.renderTimeAndDuration()}
      </div>
    `
  }

  protected renderAllDetails() {
    return html`<div class="flex flex-row items-center gap-x-3">
        <stencila-ui-node-execution-mode
          type=${this.type}
          node-id=${this.nodeId}
          value=${this.mode}
        >
        </stencila-ui-node-execution-mode>

        ${this.bounds !== undefined
          ? html`<stencila-ui-node-execution-bounds
              type=${this.type}
              node-id=${this.nodeId}
              value=${this.bounds}
            >
            </stencila-ui-node-execution-bounds>`
          : ''}
      </div>

      <div class="flex flex-row items-center gap-x-3">
        <stencila-ui-node-execution-state
          status=${this.status}
          required=${this.required}
          count=${this.count}
        >
        </stencila-ui-node-execution-state>

        ${this.count > 0
          ? html`<stencila-ui-node-execution-count
                value=${this.count}
              ></stencila-ui-node-execution-count>
              ${this.renderTimeAndDuration()}
              <stencila-ui-node-execution-bounded
                value=${this.bounded}
              ></stencila-ui-node-execution-bounded>`
          : ''}
      </div>`
  }

  protected renderTimeAndDuration() {
    return html`
      <stencila-ui-node-execution-ended
        value=${this.ended}
      ></stencila-ui-node-execution-ended>
      <stencila-ui-node-execution-duration
        value=${this.duration}
      ></stencila-ui-node-execution-duration>
    `
  }
}
