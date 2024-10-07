import { apply } from '@twind/core'
import { LitElement, html, css } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { withTwind } from '../../twind'

import { NodeChipState } from './context'

import '@shoelace-style/shoelace/dist/components/dropdown/dropdown.js'
import '@shoelace-style/shoelace/dist/components/menu/menu.js'
import '@shoelace-style/shoelace/dist/components/menu-item/menu-item.js'
import '@shoelace-style/shoelace/dist/components/divider/divider.js'
import '@shoelace-style/shoelace/dist/components/menu-label/menu-label.js'
/**
 * A menu allowing the user to control the display of the document
 * and perform actions on it.
 */
@customElement('stencila-document-menu')
@withTwind()
export class DocumentMenu extends LitElement {
  @state()
  protected open: boolean = false

  @property({ type: Boolean })
  visible: boolean = false

  @property({ type: Boolean, attribute: 'show-authorship-highlight' })
  showAuthorshipHighlight: boolean

  @property({ type: String, attribute: 'node-chip-state' })
  nodeChipState: NodeChipState

  @property({ type: Boolean, attribute: 'show-author-provenance' })
  showAuthorProvenance: boolean

  /**
   * Remove the default checked icon space.
   *
   * Make sure the divider's border-top property is set,
   * as it is being overidden by the twind base.
   */
  static override styles = css`
    sl-menu-item::part(checked-icon) {
      display: none;
    }
    sl-divider {
      border-top: solid var(--width) var(--color);
    }
  `

  private eventDispatch = (eventName: string, detail?: unknown) =>
    this.shadowRoot.dispatchEvent(
      new CustomEvent(eventName, {
        bubbles: true,
        composed: true,
        detail,
      })
    )

  protected override render() {
    const styles = apply(['fixed right-8 top-8 z-50', 'font-sans'])

    return html`
      <div class=${styles} @mouseleave=${() => (this.open = false)}>
        <sl-dropdown
          ?open=${this.open}
          @sl-hide=${() => (this.open = false)}
          class=${styles}
          placement="bottom-end"
        >
          ${this.renderMenuToggle()} ${this.renderMenu()}
        </sl-dropdown>
      </div>
    `
  }

  renderMenuToggle = () => {
    const styles = apply([
      'flex justify-center items-center',
      'ml-auto',
      'w-10 h-10',
      'drop-shadow',
      !this.open ? 'grayscale hover:grayscale-0' : '',
      'cursor-pointer',
    ])

    return html`
      <div
        class=${styles}
        slot="trigger"
        @mouseenter=${() => (this.open = true)}
      >
        <stencila-ui-icon class="text-4xl" name="stencila"></stencila-ui-icon>
      </div>
    `
  }

  renderMenu = () => {
    return html`
      <sl-menu class="mt-1 bg-gray-50 border border-gray-200">
        ${this.renderMenuItem(
          'Show article authors and provenance',
          'toggle-author-provenance',
          this.showAuthorProvenance
        )}
        ${this.renderMenuItem(
          'Show authorship highlighting',
          'toggle-authorship-highlight',
          this.showAuthorshipHighlight
        )}
        <sl-divider></sl-divider>
        <sl-menu-label>Display Node Info</sl-menu-label>
        ${this.renderMenuItem(
          'Hide All',
          'update-nodecard-state',
          this.nodeChipState === 'hidden',
          'hidden'
        )}
        ${this.renderMenuItem(
          'Show on hover',
          'update-nodecard-state',
          this.nodeChipState === 'hover-only',
          'hover-only'
        )}
        ${this.renderMenuItem(
          'Show all',
          'update-nodecard-state',
          this.nodeChipState === 'show-all',
          'show-all'
        )}
      </sl-menu>
    `
  }

  renderMenuItem(
    text: string,
    event: string,
    active: boolean,
    eventDetail?: unknown
  ) {
    return html`
      <sl-menu-item @click=${() => this.eventDispatch(event, eventDetail)}>
        <stencila-ui-icon
          name="check"
          slot="prefix"
          class="${active ? 'opacity-100' : 'opacity-0'}"
        ></stencila-ui-icon>
        ${text}
      </sl-menu-item>
    `
  }

  renderChipOptions() {
    return html`
      <div class="py-1">
        <div class="font-bold text-sm mb-1 pl-2 pr-4">Display Node Info</div>
        ${this.renderMenuItem(
          'Hide All',
          'update-nodecard-state',
          this.nodeChipState === 'hidden',
          'hidden'
        )}
        ${this.renderMenuItem(
          'Show on hover',
          'update-nodecard-state',
          this.nodeChipState === 'hover-only',
          'hover-only'
        )}
        ${this.renderMenuItem(
          'Show all',
          'update-nodecard-state',
          this.nodeChipState === 'show-all',
          'show-all'
        )}
      </div>
    `
  }
}
