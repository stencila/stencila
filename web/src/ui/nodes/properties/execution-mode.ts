import { ExecutionMode } from '@stencila/types'
import { css } from '@twind/core'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { documentCommandEvent } from '../../../clients/commands'
import { withTwind } from '../../../twind'
import { IconName } from '../../icons/icon'
import { UIBaseClass } from '../mixins/ui-base-class'

/**
 * A component for displaying/selecting the `executionMode` property of executable nodes
 */
@customElement('stencila-ui-node-execution-mode')
@withTwind()
export class UINodeExecutionMode extends UIBaseClass {
  @property()
  value?: ExecutionMode

  @property({ type: Boolean })
  disabled: boolean

  protected propertyName: 'executionMode' | 'executionRecursion' =
    'executionMode'

  /**
   * On a change to value, send a patch to update it in the document
   */
  private onChanged(value: ExecutionMode) {
    this.value = value

    this.dispatchEvent(
      documentCommandEvent({
        command: 'patch-node',
        nodeType: this.type,
        nodeIds: [this.nodeId],
        nodeProperty: [this.propertyName, value],
      })
    )
  }

  override render() {
    const { borderColour, textColour } = this.ui

    const tooltip =
      this.propertyName === 'executionMode'
        ? 'Execution mode for this node'
        : 'Execution mode for descendants of this node'

    const icon = (value: ExecutionMode): IconName => {
      switch (value) {
        case 'Default':
          return 'bracesAsterisk'
        case 'Always':
          return 'fastForwardCircle'
        case 'Auto':
          return 'lightning'
        case 'Needed':
          return 'playCircle'
        case 'Locked':
          return 'lock'
        case 'Safe':
          return 'coneStriped'
        case 'Secure':
          return 'shieldCheck'
        case 'Never':
          return 'ban'
      }
    }

    const help = (value: ExecutionMode): string => {
      switch (value) {
        case 'Default':
          return 'Use the configured default'
        case 'Always':
          return 'Always run, including on demand'
        case 'Auto':
          return 'Run automatically when stale, and on demand'
        case 'Needed':
          return 'Run when needed (e.g. when stale and document is run), and on demand'
        case 'Locked':
          return 'Do not run, even on demand'
        case 'Safe':
          return 'Only run if considered safe'
        case 'Secure':
          return 'Only run within a secure sandbox'
        case 'Never':
          return 'Do not run'
      }
    }

    const menuItemStyles = css`
      &::part(checked-icon),
      &::part(submenu-icon) {
        display: none;
      }
    `

    const alternatives: ExecutionMode[] =
      this.propertyName === 'executionMode'
        ? ['Default', 'Always', 'Auto', 'Needed', 'Locked']
        : ['Default', 'Always', 'Safe', 'Secure', 'Never']

    const menuItems = alternatives.map(
      (value: ExecutionMode) =>
        html`<sl-menu-item
          class=${menuItemStyles}
          @click=${() => this.onChanged(value)}
        >
          <div class="px-2 text-[${textColour}]">
            <div class="flex flex-row gap-2">
              <stencila-ui-icon name=${icon(value)}></stencila-ui-icon>
              <span class="text-xs">${value}</span>
            </div>
            <div class="mt-1 text-[0.65rem] opacity-70">${help(value)}</div>
          </div>
        </sl-menu-item>`
    )

    let value = this.value
    if (value.length == 0) {
      value = 'Default'
    }

    return html`
      <div class="flex flex-row gap-1 items-center">
        <sl-dropdown>
          <stencila-ui-icon-button
            name="chevronDown"
            class="text-xs text-[${textColour}]"
            slot="trigger"
          ></stencila-ui-icon-button>
          <sl-menu class="rounded border border-[${borderColour}]"
            >${menuItems}</sl-menu
          >
        </sl-dropdown>

        <sl-tooltip content=${tooltip}>
          <div class="flex flex-row gap-1 items-center">
            <stencila-ui-icon
              class="text-base"
              name=${icon(value)}
            ></stencila-ui-icon>
            ${value}
          </div>
        </sl-tooltip>
      </div>
    `
  }
}
