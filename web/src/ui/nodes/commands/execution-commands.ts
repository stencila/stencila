import '@shoelace-style/shoelace/dist/components/icon-button/icon-button'
import '@shoelace-style/shoelace/dist/components/icon/icon'
import '@shoelace-style/shoelace/dist/components/tooltip/tooltip'
import { apply, css } from '@twind/core'
import { html } from 'lit'
import { customElement } from 'lit/decorators'
import tailwindConfig from 'tailwindcss/defaultConfig'
import resolveConfig from 'tailwindcss/resolveConfig'

import {
  DocumentCommand,
  documentCommandEvent,
} from '../../../clients/commands'
import { withTwind } from '../../../twind'
import { UIBaseClass } from '../mixins/ui-base-class'

const colours = resolveConfig(tailwindConfig).theme.colors

// TODO - disable all buttons when execution is running / pending.
// TODO - refactor these menu items into reusable components to use in the preview menu as well.

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
      'flex flex-row items-center flex-shrink-0',
      `text-${this.ui.textColour}`,
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
        ${this.renderDropdown()}
      </div>
    `
  }

  renderDropdown() {
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
        &:hover {
          color: inherit;
        }
      }
    `

    const itemStyles = apply([
      'block',
      'w-full',
      'bg-white',
      'text-sm text-grey-aluminium text-left',
    ])

    const itemPartStyles = css`
      &::part(checked-icon),
      &::part(submenu-icon) {
        display: none;
      }

      &::part(base) {
        width: 100%;
        padding: 0.25rem 1rem;
        &:hover {
          background-color: ${colours['gray'][200]};
        }
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
        <sl-icon-button
          class="text-xs ${buttonStyles} hover:contrast-[105%]"
          name="chevron-down"
          slot="trigger"
        ></sl-icon-button>
        <sl-menu class="z-50">
          <sl-menu-item
            class="${itemStyles} ${itemPartStyles}"
            @click=${(e: Event) =>
              this.emitEvent(e, 'plus-upstream-downstream')}
          >
            <div class="flex items-center">
              <sl-icon name="skip-start" class="mr-1"></sl-icon>
              Run all above
            </div>
          </sl-menu-item>
          <sl-menu-item
            class="${itemStyles} ${itemPartStyles}"
            @click=${(e: Event) => this.emitEvent(e, 'plus-after')}
          >
            <div class="flex items-center">
              <sl-icon name="skip-end" class="mr-1"></sl-icon>
              Run all below
            </div>
          </sl-menu-item>
        </sl-menu>
      </sl-dropdown>
    `
  }
}
