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
import './is-echoed'
import './is-hidden'

/**
 * A component for displaying various execution related property of executable nodes
 */
@customElement('stencila-ui-node-execution-details')
@withTwind()
export class UINodeExecutionDetails extends UIBaseClass {
  @property()
  mode?: ExecutionMode

  @property()
  bounds?: ExecutionBounds

  @property({ attribute: 'is-echoed' })
  isEchoed?: string

  @property({ attribute: 'is-hidden' })
  isHidden?: string

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
    const { colour, borderColour, textColour } = this.ui

    const classes = apply([
      'flex flex-row flex-wrap items-center justify-between gap-3',
      `text-[${textColour}] text-xs leading-tight`,
      'min-h-[2.25rem]',
      'py-1.5 px-4',
      `bg-[${colour}]`,
      `border-t border-[${borderColour}]`,
      'font-sans',
      'select-none',
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
        ${this.isEchoed !== undefined
          ? html`<stencila-ui-node-is-echoed
              type=${this.type}
              node-id=${this.nodeId}
              ?value=${this.isEchoed == 'true'}
            >
            </stencila-ui-node-is-echoed>`
          : ''}
        ${this.isHidden !== undefined
          ? html`<stencila-ui-node-is-hidden
              type=${this.type}
              node-id=${this.nodeId}
              ?value=${this.isHidden == 'true'}
            >
            </stencila-ui-node-is-hidden>`
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
