import { ExecutionBounds } from '@stencila/types'
import { css } from '@twind/core'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import { documentCommandEvent } from '../../../clients/commands'
import { withTwind } from '../../../twind'
import { IconName } from '../../icons/icon'
import { UIBaseClass } from '../mixins/ui-base-class'

/**
 * A component for displaying/selecting the `executionBounds` property of executable nodes
 */
@customElement('stencila-ui-node-execution-bounds')
@withTwind()
export class UINodeExecutionBounds extends UIBaseClass {
  @property()
  value?: ExecutionBounds

  /**
   * On a change to value, send a patch to update it in the document
   */
  private onChanged(value: ExecutionBounds) {
    this.value = value

    this.dispatchEvent(
      documentCommandEvent({
        command: 'patch-node',
        nodeType: this.type,
        nodeIds: [this.nodeId],
        nodeProperty: ['executionBounds', value],
      })
    )
  }

  override render() {
    const { borderColour, textColour } = this.ui

    const icon = (value: ExecutionBounds): IconName => {
      switch (value) {
        case 'Default':
          return 'asterisk'
        case 'Main':
          return 'arrowUpCircle'
        case 'Fork':
          return 'arrowRampRight3'
        case 'Limit':
          return 'coneStriped'
        case 'Box':
          return 'box'
        case 'Skip':
          return 'ban'
      }
    }

    const help = (value: ExecutionBounds): string => {
      switch (value) {
        case 'Default':
          return 'Use the configured default'
        case 'Main':
          return 'Execute within the main set of kernels'
        case 'Fork':
          return 'Execute within a forked set of kernels'
        case 'Limit':
          return 'Execute within a forked set of kernels with limited capabilities'
        case 'Box':
          return 'Execute within a forked set of kernels within a sandbox'
        case 'Skip':
          return 'Skip execution'
      }
    }

    const menuItemStyles = css`
      &::part(checked-icon),
      &::part(submenu-icon) {
        display: none;
      }
    `

    const alternatives: ExecutionBounds[] = [
      'Default',
      'Main',
      'Fork',
      'Limit',
      'Box',
      'Skip',
    ]

    const menuItems = alternatives.map(
      (value: ExecutionBounds) =>
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

        <sl-tooltip content="Execution bounds within this node">
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
