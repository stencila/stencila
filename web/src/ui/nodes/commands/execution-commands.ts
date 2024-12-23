import '@shoelace-style/shoelace/dist/components/tooltip/tooltip'
import { apply, css } from '@twind/core'
import { html } from 'lit'
import { customElement, property } from 'lit/decorators'

import {
  DocumentCommand,
  documentCommandEvent,
} from '../../../clients/commands'
import { withTwind } from '../../../twind'
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
  @property({ type: Boolean })
  hideDropDown: boolean = false

  /**
   * Emit a custom event to execute the document with this
   * node id and command scope
   */
  private emitEvent(e: Event, scope: DocumentCommand['scope']) {
    e.stopImmediatePropagation()

    this.dispatchEvent(
      documentCommandEvent({
        command: 'execute-nodes',
        nodeType: this.type,
        nodeIds: [this.nodeId],
        scope,
      })
    )
  }

  override render() {
    const classes = apply([
      'flex flex-row items-center flex-shrink-0',
      `text-${this.ui.textColour}`,
    ])

    return html`
      <div class=${classes}>
        ${!this.hideDropDown ? this.renderDropdown() : ''}

        <sl-tooltip content="Run this node">
          <stencila-ui-icon-button
            name="play"
            class="text-2xl"
            @click=${(e: Event) => this.emitEvent(e, 'only')}
          ></stencila-ui-icon-button>
        </sl-tooltip>

        <slot></slot>
      </div>
    `
  }

  renderDropdown() {
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
        @click=${(e: Event) => e.stopImmediatePropagation()}
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
            @click=${(e: Event) => this.emitEvent(e, 'plus-before')}
          >
            <div class="flex items-center gap-1 text-[${textColour}]">
              <stencila-ui-icon name="skipStart"></stencila-ui-icon>
              Run all above, then this
            </div>
          </sl-menu-item>
          <sl-menu-item
            class="${itemStyles} ${itemPartStyles}"
            @click=${(e: Event) => this.emitEvent(e, 'plus-after')}
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
