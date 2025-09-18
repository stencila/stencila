import { consume } from '@lit/context'
import { apply } from '@twind/core'
import { LitElement, html, css } from 'lit'
import { customElement, state } from 'lit/decorators'

import { Entity } from '../../nodes/entity'
import { withTwind } from '../../twind'
import { UIBlockOnDemand } from '../nodes/cards/block-on-demand'
import { UIInlineOnDemand } from '../nodes/cards/inline-on-demand'

import { documentContext, DocumentContext, NodeMarkerState } from './context'

import '@shoelace-style/shoelace/dist/components/dropdown/dropdown.js'
import '@shoelace-style/shoelace/dist/components/menu/menu.js'
import '@shoelace-style/shoelace/dist/components/menu-item/menu-item.js'
import '@shoelace-style/shoelace/dist/components/divider/divider.js'
import '@shoelace-style/shoelace/dist/components/menu-label/menu-label.js'

/**
 * Color scheme preferences for the document
 */
type ColorScheme = 'system' | 'light' | 'dark'

/**
 * Utility functions for color scheme management
 */
class ColorSchemeManager {
  static loadColorSchemePreference(): ColorScheme {
    try {
      const saved = localStorage.getItem('stencila-color-scheme-preference') as ColorScheme
      return saved && ['system', 'light', 'dark'].includes(saved) ? saved : 'system'
    } catch (e) {
      return 'system'
    }
  }

  static applyColorScheme(colorScheme: ColorScheme) {
    const root = document.documentElement

    if (colorScheme === 'system') {
      root.removeAttribute('data-color-scheme')
    } else {
      root.setAttribute('data-color-scheme', colorScheme)
    }
  }

  static persistColorScheme(colorScheme: ColorScheme) {
    try {
      localStorage.setItem('stencila-color-scheme-preference', colorScheme)
    } catch (e) {
      console.warn('Could not persist color-scheme preference:', e)
    }
  }
}

// Apply saved color scheme immediately to avoid flash
ColorSchemeManager.applyColorScheme(ColorSchemeManager.loadColorSchemePreference())

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
  protected get nodeMarkerState(): NodeMarkerState {
    return this.context?.nodeMarkerState ?? 'hidden'
  }

  @state()
  protected get showAuthorProvenance(): boolean {
    return this.context?.showAuthorProvenance ?? false
  }

  @state()
  protected colorScheme: ColorScheme = 'system'

  @state()
  protected open: boolean = false

  /**
   * Get the current effective color scheme (resolving 'system' to actual preference)
   */
  private getEffectiveColorScheme(): 'light' | 'dark' {
    if (this.colorScheme === 'system') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
    }
    return this.colorScheme
  }


  /**
   * Initialize color scheme on component connection
   */
  override connectedCallback() {
    super.connectedCallback()

    // Load saved color scheme preference for component state
    // (Theme already applied at module load to prevent flash)
    this.colorScheme = ColorSchemeManager.loadColorSchemePreference()
  }

  /**
   * Cycle through color scheme options: system → light → dark → system
   */
  private cycleColorScheme() {
    const schemes: ColorScheme[] = ['system', 'light', 'dark']
    const currentIndex = schemes.indexOf(this.colorScheme)
    const nextScheme = schemes[(currentIndex + 1) % schemes.length]

    // Update local state
    this.colorScheme = nextScheme

    // Apply to DOM and persist
    ColorSchemeManager.applyColorScheme(nextScheme)
    ColorSchemeManager.persistColorScheme(nextScheme)
  }

  /**
   * Find all instances of the stencila node Entities,
   *
   * then finds each card element and triggers the public "cardOpen|cardClose" method
   * based on the action.
   */
  private nodeCardToggle(action: 'expand' | 'collapse') {
    const stencilaNodes = Array.from(document.querySelectorAll('*')).filter(
      (element) => {
        return (
          element.tagName.toLowerCase().startsWith('stencila-') &&
          element instanceof Entity
        )
      }
    )
    stencilaNodes.forEach((el) => {
      const card = el.shadowRoot.querySelector(
        'stencila-ui-block-on-demand, stencila-ui-inline-on-demand'
      ) as UIBlockOnDemand | UIInlineOnDemand
      if (card) {
        if (action === 'expand') {
          card.openCard()
        } else if (action === 'collapse') {
          card.closeCard()
        }
      }
    })
  }

  /**
   * Make sure the divider's border-top property is set,
   * this is being overridden by the twind base stylesheet.
   * +
   * Reduce the y padding of the sl menu components.
   */
  static override styles = css`
    sl-divider {
      border-top: solid var(--width) var(--color);
      margin: 0.25rem 0;
    }
    sl-menu-item::part(base),
    sl-menu-label::part(base) {
      padding: 0.125rem var(--sl-spacing-2x-small);
    }
  `

  /**
   * custom event dispatch to update the document context based on menu item selection
   */
  private eventDispatch = (eventName: string, detail?: unknown) => {
    this.shadowRoot.dispatchEvent(
      new CustomEvent(eventName, {
        bubbles: true,
        composed: true,
        detail,
      })
    )
  }

  /**
   * Handle the shoelace `sl-select` event for the menu
   */
  handleSelect(event: CustomEvent) {
    const selectedItem = event.detail.item
    if (selectedItem) {
      const eventName = selectedItem.getAttribute('data-event')
      if (eventName) {
        if (eventName === 'update-nodemarker-state') {
          const value = selectedItem.getAttribute('value')
          this.eventDispatch(eventName, value)
        } else {
          this.eventDispatch(eventName)
        }
      }
    }
  }

  protected override render() {
    const styles = apply(['fixed right-8 top-8 z-50', 'font-sans'])

    return html`
      <div class=${styles} @mouseleave=${() => (this.open = false)}>
        <sl-dropdown
          ?open=${this.open}
          @sl-hide=${() => (this.open = false)}
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
      !this.open ? 'grayscale' : '',
      'cursor-pointer',
    ])

    return html`
      <div
        class=${styles}
        slot="trigger"
        @mouseenter=${() => (this.open = true)}
      >
        <stencila-ui-icon
          class="text-xl"
          name="stencilaColor"
        ></stencila-ui-icon>
      </div>
    `
  }

  renderMenu = () => {
    const effectiveColorScheme = this.getEffectiveColorScheme()
    const colorSchemeIcon = effectiveColorScheme === 'dark' ? 'moon' : 'sun'

    // Calculate next scheme in cycle
    const schemes: ColorScheme[] = ['system', 'light', 'dark']
    const currentIndex = schemes.indexOf(this.colorScheme)
    const nextScheme = schemes[(currentIndex + 1) % schemes.length]

    // Format current scheme display
    const currentDisplay = this.colorScheme === 'system'
      ? `Auto (${effectiveColorScheme})`
      : this.colorScheme.charAt(0).toUpperCase() + this.colorScheme.slice(1)

    // Format next scheme display
    const nextDisplay = nextScheme === 'system'
      ? `Auto (${window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'})`
      : nextScheme.charAt(0).toUpperCase() + nextScheme.slice(1)

    const colorSchemeLabel = `${currentDisplay} → ${nextDisplay}`

    return html`
      <sl-menu
        class="mt-1 bg-gray-50 border border-gray-200"
        id="preview-menu"
        @sl-select=${this.handleSelect}
      >
        <sl-menu-label>
          <div class="flex items-center gap-2">Appearance</div>
        </sl-menu-label>
        <sl-menu-item @click=${() => this.cycleColorScheme()}>
          <stencila-ui-icon name="${colorSchemeIcon}" slot="prefix"></stencila-ui-icon>
          <span class="text-sm">${colorSchemeLabel}</span>
        </sl-menu-item>
        <sl-divider></sl-divider>
        <sl-menu-label>
          <div class="flex items-center gap-2">Document</div>
        </sl-menu-label>
        ${
          /*
        TODO: Consider reinstating this
        <sl-menu-item type="checkbox" data-event="toggle-author-provenance">
          <stencila-ui-icon name="feather" slot="prefix"></stencila-ui-icon>
          <span class="text-sm">Show authors and provenance</span>
        </sl-menu-item>
        */ ''
        }
        <sl-menu-item type="checkbox" data-event="toggle-authorship-highlight">
          <stencila-ui-icon name="highlights" slot="prefix"></stencila-ui-icon>
          <span class="text-sm">Show authorship highlighting</span>
        </sl-menu-item>
        <sl-divider></sl-divider>
        <sl-menu-label>
          <div class="flex items-center gap-2">Node Markers</div>
        </sl-menu-label>
        <sl-menu-item
          type="checkbox"
          data-event="update-nodemarker-state"
          value="hover-only"
          ?checked=${this.nodeMarkerState === 'hover-only'}
        >
          <stencila-ui-icon name="cursor" slot="prefix"></stencila-ui-icon>
          <span class="text-sm">Show on hover</span>
        </sl-menu-item>
        <sl-menu-item
          type="checkbox"
          data-event="update-nodemarker-state"
          value="show-all"
          ?checked=${this.nodeMarkerState === 'show-all'}
        >
          <stencila-ui-icon name="eye" slot="prefix"></stencila-ui-icon>
          <span class="text-sm">Show all</span>
        </sl-menu-item>
        <sl-menu-item
          type="checkbox"
          data-event="update-nodemarker-state"
          value="hidden"
          ?checked=${this.nodeMarkerState === 'hidden'}
        >
          <stencila-ui-icon name="eyeSlash" slot="prefix"></stencila-ui-icon>
          <span class="text-sm">Hide All</span>
        </sl-menu-item>
        <sl-divider></sl-divider>
        <sl-menu-label>
          <div class="flex items-center gap-2">Node Cards</div>
        </sl-menu-label>
        <sl-menu-item @click=${() => this.nodeCardToggle('expand')}>
          <stencila-ui-icon
            name="arrowsExpand"
            slot="prefix"
          ></stencila-ui-icon>
          <span class="text-sm">Expand all</span>
        </sl-menu-item>
        <sl-menu-item @click=${() => this.nodeCardToggle('collapse')}>
          <stencila-ui-icon
            name="arrowsCollapse"
            slot="prefix"
          ></stencila-ui-icon>
          <span class="text-sm">Collapse all</span>
        </sl-menu-item>
      </sl-menu>
    `
  }
}
