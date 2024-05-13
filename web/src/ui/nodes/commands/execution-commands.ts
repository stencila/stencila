import '@shoelace-style/shoelace/dist/components/icon-button/icon-button'
import '@shoelace-style/shoelace/dist/components/tooltip/tooltip'
import { apply, css } from '@twind/core'
import { html } from 'lit'
import { customElement } from 'lit/decorators'

import {
  DocumentCommand,
  documentCommandEvent,
} from '../../../clients/commands'
import { withTwind } from '../../../twind'
import { UIBaseClass } from '../mixins/ui-base-class'

/**
 * A component for providing common execution related actions of executable nodes
 */
@customElement('stencila-ui-node-execution-commands')
@withTwind()
export class UINodeExecutionCommands extends UIBaseClass {
  /**
   * Emit a custom event to execute the document with this
   * node id and command scope
   */
  private emitEvent(e: Event, scope: DocumentCommand['scope']) {
    e.stopImmediatePropagation()

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
      'flex flex-row items-center gap-x-3 flex-shrink-0',
      'text-black',
    ])

    const dividerClasses = apply([
      'h-4 w-0',
      `border-l-2 border-[${this.ui.borderColour}]`,
      `brightness-75`,
    ])

    const buttonClasses = css`
      &::part(base) {
        --sl-spacing-x-small: 0;
      }
    `

    return html`
      <div class=${containerClasses}>
        <sl-tooltip content="Execute this node">
          <sl-icon-button
            name="play"
            library="stencila"
            @click=${(e: Event) => {
              this.emitEvent(e, 'only')
            }}
            class=${`${buttonClasses} text-base`}
          ></sl-icon-button>
        </sl-tooltip>

        <div class=${dividerClasses} aria-hidden="true"></div>

        <sl-tooltip content="Execute this node and all following nodes">
          <sl-icon-button
            name="skip"
            library="stencila"
            @click=${(e: Event) => this.emitEvent(e, 'plus-after')}
            class=${`${buttonClasses} text-2xl`}
          ></sl-icon-button>
        </sl-tooltip>

        <div class=${dividerClasses}></div>

        <sl-tooltip
          content="Execute any stale upstream dependencies, this node, and any downstream dependant nodes. Coming soon!"
        >
          <sl-icon-button
            name="deps-tree"
            library="stencila"
            disabled
            @click=${(e: Event) =>
              this.emitEvent(e, 'plus-upstream-downstream')}
            class=${`${buttonClasses} text-xl`}
          ></sl-icon-button>
        </sl-tooltip>
      </div>
    `
  }
}
