import {
  AutomaticExecution,
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

  @property({ attribute: 'auto-exec' })
  autoExec?: AutomaticExecution

  @property({ type: Array })
  tags?: ExecutionTag[]

  @property({ type: Number })
  count?: number

  @property()
  required?: ExecutionRequired

  @property()
  status?: ExecutionStatus

  @property({ type: Number })
  ended?: number

  @property({ type: Number })
  duration?: number

  @property({ attribute: 'header-bg' })
  headerBg: string | undefined = undefined

  @property()
  display: 'inline' | 'block' = 'block'

  override render() {
    const { borderColour } = nodeUi(this.type)

    const baseClasses = [
      'flex flex-col gap-3',
      'text-xs leading-tight',
      'font-sans',
    ]

    const blockClasses =
      this.display === 'block'
        ? [
            'py-1.5 px-4',
            `bg-[${borderColour}]`,
            'border-t border-b border-black/20',
          ]
        : []

    const classes = apply([...baseClasses, ...blockClasses])

    return html`
      <div class="@container">
        <div class=${`${classes} @[30rem]:flex-row`}>
          <stencila-ui-node-execution-state
            status=${this.status}
            required=${this.required}
          ></stencila-ui-node-execution-state>
          <stencila-ui-node-execution-count
            value=${this.count}
          ></stencila-ui-node-execution-count>
          <stencila-ui-node-execution-ended
            value=${this.ended}
          ></stencila-ui-node-execution-ended>
          <stencila-ui-node-execution-duration
            value=${this.duration}
          ></stencila-ui-node-execution-duration>
        </div>
      </div>
    `
  }
}
