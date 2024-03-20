import {
  AutomaticExecution,
  ExecutionRequired,
  ExecutionStatus,
  ExecutionTag,
  NodeType,
} from '@stencila/types'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { withTwind } from '../../../twind'

import './generic/collapsible-details'
import './execution-count'
import './execution-duration'
import './execution-ended'
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

  override render() {
    return html`
      <stencila-ui-node-collapsible-details
        type=${this.type}
        title="Details"
        icon-name="info"
        wrapper-css="border-t border-black/30"
        ?collapsed=${true}
      >
        <div class="flex flex-col gap-y-3">
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
      </stencila-ui-node-collapsible-details>
    `
  }
}
