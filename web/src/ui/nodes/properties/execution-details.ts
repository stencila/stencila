import {
  ExecutionKind,
  ExecutionMode,
  ExecutionRequired,
  ExecutionStatus,
  ExecutionTag,
  NodeType,
} from '@stencila/types'
import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import './execution-count'
import './execution-duration'
import './execution-ended'
import './execution-kind'
import './execution-state'

import { withTwind } from '../../../twind'
import { nodeUi } from '../icons-and-colours'

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
export class UINodeExecutionDetails extends LitElement {
  @property()
  type: NodeType

  @property()
  mode?: ExecutionMode

  @property({ type: Array })
  tags?: ExecutionTag[]

  @property({ type: Number })
  count?: number

  @property()
  required?: ExecutionRequired = 'NeverExecuted'

  @property()
  status?: ExecutionStatus

  @property()
  kind?: ExecutionKind

  @property({ type: Number })
  ended?: number

  @property({ type: Number })
  duration?: number

  override render() {
    const { colour, borderColour } = nodeUi(this.type)

    const classes = apply([
      'flex flex-row flex-wrap gap-3',
      'text-xs leading-tight',
      'min-h-[2.25rem]',
      'py-1.5 px-4',
      `bg-[${colour}]`,
      `border-t border-[${borderColour}]`,
      'font-sans',
    ])

    return html`
      <div class="@container">
        <div class=${`${classes}`}>
          ${this.type !== 'SuggestionBlock'
            ? this.renderAllDetails()
            : this.renderTimeAndDuration()}
        </div>
      </div>
    `
  }

  protected renderAllDetails() {
    return html`<stencila-ui-node-execution-state
        status=${this.status}
        required=${this.required}
        count=${this.count}
      ></stencila-ui-node-execution-state>
      ${this.count > 0
        ? html`<stencila-ui-node-execution-count
              value=${this.count}
            ></stencila-ui-node-execution-count>
            ${this.renderTimeAndDuration()}
            <stencila-ui-node-execution-kind
              value=${this.kind}
            ></stencila-ui-node-execution-kind>`
        : ''}`
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
