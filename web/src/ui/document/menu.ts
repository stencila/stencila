import { consume } from '@lit/context'
import { apply } from '@twind/core'
import { LitElement, html, css } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { withTwind } from '../../twind'

import { documentContext, DocumentContext, NodeChipState } from './context'

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
  @consume({ context: documentContext, subscribe: true })
  @state()
  protected context: DocumentContext

  @state()
  protected get showAuthorshipHighlight(): boolean {
    return this.context?.showAllAuthorshipHighlight ?? false
  }

  @state()
  protected get nodeChipState(): NodeChipState {
    return this.context?.nodeChipState ?? 'hidden'
  }

  @state()
  protected get showAuthorProvenance(): boolean {
    return this.context?.showAuthorProvenance ?? false
  }

  @state()
  protected open: boolean = false

  @property({ type: Boolean })
  visible: boolean = false

  /**
   * Remove the default checked icon space.
   *
   * Make sure the divider's border-top property is set,
   * as it is being overidden by the twind base.
   */
  static override styles = css`
    sl-divider {
      border-top: solid var(--width) var(--color);
    }
  `

  private eventDispatch = (eventName: string, detail?: unknown) => {
    this.shadowRoot.dispatchEvent(
      new CustomEvent(eventName, {
        bubbles: true,
        composed: true,
        detail,
      })
    )
  }

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

  handleSelect(event: CustomEvent) {
    const selectedItem = event.detail.item
    if (selectedItem) {
      const eventName = selectedItem.getAttribute('data-event')
      if (eventName) {
        if (eventName === 'update-nodecard-state') {
          const value = selectedItem.getAttribute('value')
          this.eventDispatch(eventName, value)
        } else {
          this.eventDispatch(eventName)
        }
      }
    }
  }

  renderMenu = () => {
    return html`
      <sl-menu
        class="mt-1 bg-gray-50 border border-gray-200"
        id="preview-menu"
        @sl-select=${this.handleSelect}
      >
        <sl-menu-item type="checkbox" data-event="toggle-author-provenance">
          <stencila-ui-icon name="feather" slot="prefix"></stencila-ui-icon>
          ${'Show article authors and provenance'}
        </sl-menu-item>
        <sl-menu-item type="checkbox" data-event="toggle-authorship-highlight">
          <stencila-ui-icon name="highlights" slot="prefix"></stencila-ui-icon>
          ${'Show authorship highlighting'}
        </sl-menu-item>
        <sl-divider></sl-divider>
        <sl-menu-label>
          <div class="flex items-center gap-2 text-lg">
            <stencila-ui-icon
              name="chip"
              slot="prefix"
              class="text-2xl"
            ></stencila-ui-icon>
            Display Node Chips
          </div>
        </sl-menu-label>
        <sl-menu-item
          type="checkbox"
          data-event="update-nodecard-state"
          value="hidden"
          ?checked=${this.nodeChipState === 'hidden'}
        >
          <stencila-ui-icon name="eyeSlash" slot="prefix"></stencila-ui-icon>
          ${'Hide All'}
        </sl-menu-item>
        <sl-menu-item
          type="checkbox"
          data-event="update-nodecard-state"
          value="hover-only"
          ?checked=${this.nodeChipState === 'hover-only'}
        >
          <stencila-ui-icon name="cursor" slot="prefix"></stencila-ui-icon>
          ${'Show on hover'}
        </sl-menu-item>
        <sl-menu-item
          type="checkbox"
          data-event="update-nodecard-state"
          value="show-all"
          ?checked=${this.nodeChipState === 'show-all'}
        >
          <stencila-ui-icon name="eye" slot="prefix"></stencila-ui-icon>
          ${'Show all'}
        </sl-menu-item>
      </sl-menu>
    `
  }
}
