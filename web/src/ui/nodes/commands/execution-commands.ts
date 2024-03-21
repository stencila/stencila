import '@shoelace-style/shoelace/dist/components/icon-button/icon-button'
import '@shoelace-style/shoelace/dist/components/tooltip/tooltip'
import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property } from 'lit/decorators'

import {
  DocumentCommand,
  documentCommandEvent,
} from '../../../clients/commands'
import { withTwind } from '../../../twind'
import { NodeId } from '../../../types'

/**
 * A component for providing common execution related actions of executable nodes
 */
@customElement('stencila-ui-node-execution-commands')
@withTwind()
export class UINodeExecutionCommands extends LitElement {
  /**
   * The id of the node that these commands apply to
   */
  @property({ attribute: 'node-id' })
  nodeId: NodeId

  /**
   * Emit a custom event to execute the document with this
   * node id a command scope
   */
  private emitEvent(scope: DocumentCommand['scope']) {
    this.dispatchEvent(
      documentCommandEvent({
        command: 'execute-nodes',
        nodeIds: [this.nodeId],
        scope,
      })
    )
  }

  override render() {
    const containerClasses = apply([
      'flex flex-row items-center gap-x-2',
      'text-black',
    ])

    const dividerClasses = apply([
      'h-4 w-0',
      'border border-gray-400',
      'mix-blend-multiply opacity-50',
    ])

    return html`
      <div class=${containerClasses}>
        <sl-tooltip content="Execute this node">
          <sl-icon-button
            name="play"
            library="stencila"
            @click=${() => this.emitEvent('only')}
          ></sl-icon-button>
        </sl-tooltip>

        <div class=${dividerClasses}></div>

        <sl-tooltip content="Execute this node and all following nodes">
          <sl-icon-button
            name="skip"
            library="stencila"
            class="text-2xl"
            @click=${() => this.emitEvent('plus-after')}
          ></sl-icon-button>
        </sl-tooltip>

        <div class=${dividerClasses}></div>

        <sl-tooltip
          content="Execute any stale upstream dependencies, this node, and any downstream dependant nodes. Coming soon!"
        >
          <sl-icon-button
            name="deps-tree"
            library="stencila"
            class="text-xl"
            disabled
            @click=${() => this.emitEvent('plus-upstream-downstream')}
          ></sl-icon-button>
        </sl-tooltip>
      </div>
    `
  }
}
