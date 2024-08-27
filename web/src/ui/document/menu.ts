import { apply } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property, state } from 'lit/decorators'

import { withTwind } from '../../twind'

import { NodeChipState } from './context'

import '../buttons/icon'

@customElement('preview-menu')
@withTwind()
export class DocumentViewMenu extends LitElement {
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

  private eventDispatch = (eventName: string, detail?: unknown) =>
    this.shadowRoot.dispatchEvent(
      new CustomEvent(eventName, {
        bubbles: true,
        composed: true,
        detail,
      })
    )

  protected override render() {
    const styles = apply([
      'fixed right-8 top-8 z-50',
      'font-sans',
      !this.visible && 'opacity-0',
      !this.visible && 'pointer-events-none',
    ])

    return html`
      <div class=${styles}>${this.renderMenuToggle()} ${this.renderMenu()}</div>
    `
  }

  renderMenuToggle = () => {
    const styles = apply([
      'ml-auto',
      'block',
      'bg-gray-100',
      'border rounded',
      'drop-shadow-xl',
    ])

    return html`
      <button class=${styles} @click=${() => (this.open = !this.open)}>
        <div
          class="flex justify-center items-center w-8 h-8 hover:text-gray-400"
        >
          <stencila-ui-icon
            name=${this.open ? 'chevronDown' : 'bars'}
          ></stencila-ui-icon>
        </div>
      </button>
    `
  }

  renderMenu = () => {
    const styles = apply([
      this.open ? 'opacity-100' : 'opacity-0',
      this.open ? 'max-w-300 max-h-500' : 'max-w-0 max-h-0',
      'mt-2',
      'bg-gray-100',
      'drop-shadow-xl',
      'border rounded',
      'transition-all duration-200',
      'overflow-hidden',
      !this.open && 'pointer-events-none',
    ])

    return html`
      <div class=${styles}>
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
        ${this.renderChipOptions()}
      </div>
    `
  }

  renderMenuItem(
    text: string,
    event: string,
    active: boolean,
    eventDetail?: unknown
  ) {
    const styles = apply([
      'flex items-center justify-between',
      'px-4 py-1',
      'cursor-pointer',
      'hover:bg-gray-300',
    ])

    return html`
      <div
        class=${styles}
        @click=${() => this.eventDispatch(event, eventDetail)}
      >
        <span class="leading-none text-sm mr-2">${text}</span>
        <stencila-ui-icon
          name="check"
          class="text-sm ${active ? 'opacity-100' : 'opacity-0'}"
        ></stencila-ui-icon>
      </div>
    `
  }

  renderChipOptions() {
    return html`
      <div class="py-1">
        <div class="font-bold text-sm mb-1 px-4">Display Node Info</div>
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
