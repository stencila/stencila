import '@shoelace-style/shoelace/dist/components/tooltip/tooltip'
import { apply, css } from '@twind/core'
import { html } from 'lit'
import { customElement } from 'lit/decorators'

import { runNode } from '../../../clients/commands'
import { withTwind } from '../../../twind'
import { closestGlobally } from '../../../utilities/closestGlobally'
import { UIBaseClass } from '../mixins/ui-base-class'

import '../../buttons/icon'

// TODO - disable all buttons when execution is running / pending.
// TODO - refactor these menu items into reusable components to use in the preview menu as well.

/**
 * A component for providing common execution related actions of executable nodes
 */
@customElement('stencila-ui-node-execution-commands')
@withTwind()
export class UINodeExecutionCommands extends UIBaseClass {
  private onRun(event: Event) {
    event.stopImmediatePropagation()

    this.dispatchEvent(runNode(this.type, this.nodeId))
  }

  private onRunAbove(event: Event) {
    event.stopImmediatePropagation()

    this.dispatchEvent(runNode(this.type, this.nodeId, 'plus-before'))
  }

  private onRunBelow(event: Event) {
    event.stopImmediatePropagation()

    this.dispatchEvent(runNode(this.type, this.nodeId, 'plus-after'))
  }

  override render() {
    const classes = apply([
      'flex flex-row items-center flex-shrink-0',
      `text-${this.ui.textColour}`,
    ])

    const showDropdown =
      this.depth > 0 && closestGlobally(this, 'stencila-chat-message') === null

    return html`
      <div class=${classes}>
        ${showDropdown ? this.renderDropdown() : ''}

        <sl-tooltip content="Run this node">
          <stencila-ui-icon-button
            name="play"
            class="text-2xl"
            @click=${this.onRun}
          ></stencila-ui-icon-button>
        </sl-tooltip>

        <slot></slot>
      </div>
    `
  }

  private renderDropdown() {
    const { borderColour, textColour } = this.ui

    const containerStyles = css`
      &[open] {
        & [slot='trigger'] {
          transform: rotate(180deg);
        }
      }
      &::part(panel) {
        position: relative;
        z-index: 1000;
      }
    `

    const buttonStyles = css`
      &::part(base) {
        color: ${textColour};
        &:hover {
          color: inherit;
        }
      }
    `

    const itemStyles = apply([
      'block',
      'w-full',
      'bg-white',
      'text-sm text-left',
    ])

    const itemPartStyles = css`
      &::part(checked-icon),
      &::part(submenu-icon) {
        display: none;
      }

      &::part(base) {
        width: 100%;
        padding: 0.25rem 1rem;
      }

      &::part(label) {
        font-size: 0.75rem;
      }
    `

    return html`
      <sl-dropdown
        class=${containerStyles}
        @click=${(event: Event) => event.stopImmediatePropagation()}
        placement="bottom-end"
        hoist
      >
        <stencila-ui-icon-button
          name="chevronDown"
          class="text-xs ${buttonStyles}"
          slot="trigger"
        ></stencila-ui-icon-button>

        <sl-menu class="rounded border border-[${borderColour}] z-50">
          <sl-menu-item
            class="${itemStyles} ${itemPartStyles}"
            @click=${this.onRunAbove}
          >
            <div class="flex items-center gap-1 text-[${textColour}]">
              <stencila-ui-icon name="skipStart"></stencila-ui-icon>
              Run all above, then this
            </div>
          </sl-menu-item>
          <sl-menu-item
            class="${itemStyles} ${itemPartStyles}"
            @click=${this.onRunBelow}
          >
            <div class="flex items-center text-[${textColour}]">
              <stencila-ui-icon name="skipEnd"></stencila-ui-icon>
              Run this, then all below
            </div>
          </sl-menu-item>
        </sl-menu>
      </sl-dropdown>
    `
  }
}
