import { consume } from '@lit/context'
import { apply } from '@twind/core'
import { LitElement, css, html } from 'lit'
import { customElement, state } from 'lit/decorators'

import { Entity } from '../../nodes/entity'
import { withTwind } from '../../twind'
import { UIBlockOnDemand } from '../nodes/cards/block-on-demand'
import { UIInlineOnDemand } from '../nodes/cards/inline-on-demand'

import { ColorScheme, ColorSchemeManager } from './color-scheme'
import { DocumentContext, NodeMarkerState, documentContext } from './context'
import { Theme, ThemeManager } from './themes'

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
  protected get nodeMarkerState(): NodeMarkerState {
    return this.context?.nodeMarkerState ?? 'hidden'
  }

  @state()
  protected get showAuthorProvenance(): boolean {
    return this.context?.showAuthorProvenance ?? false
  }

  @state()
  // Placeholder value, actual value loaded from ColorSchemeManager in connectedCallback
  protected colorScheme: ColorScheme = 'light'

  @state()
  protected theme: Theme = 'stencila'

  @state()
  protected open: boolean = false

  /**
   * Get the custom theme info if one is available (workspace or user)
   */
  private getCustomThemeInfo(): { type: 'workspace' | 'user', name: string } | null {
    const themeStyle = document.querySelector('style[data-theme-style]')
    if (!themeStyle) {
      return null
    }

    const themeType = themeStyle.getAttribute('data-theme-type')
    if (themeType !== 'workspace' && themeType !== 'user') {
      return null
    }

    // Get the theme name from meta tag
    const themeName = document.querySelector('meta[name="stencila-initial-theme-name"]')?.getAttribute('content')

    return {
      type: themeType,
      name: themeName || themeType
    }
  }

  /**
   * Initialize color scheme and theme on component connection
   */
  override connectedCallback() {
    super.connectedCallback()

    // Load saved preferences for component state
    // (Theme and color scheme already applied at module load to prevent flash)
    this.colorScheme = ColorSchemeManager.loadColorSchemePreference()
    this.theme = ThemeManager.loadThemePreference()
  }

  /**
   * Toggle between light and dark color schemes
   */
  private toggleColorScheme() {
    const newScheme: ColorScheme = this.colorScheme === 'light' ? 'dark' : 'light'

    // Update local state
    this.colorScheme = newScheme

    // Persist first so event listeners read the correct value, then apply
    ColorSchemeManager.persistColorScheme(newScheme)
    ColorSchemeManager.applyColorScheme(newScheme)
  }

  /**
   * Change the theme
   */
  private changeTheme(newTheme: Theme) {
    // Update local state
    this.theme = newTheme

    // Apply to DOM and persist
    ThemeManager.applyTheme(newTheme)
    ThemeManager.persistTheme(newTheme)
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
    const colorSchemeIcon = this.colorScheme === 'dark' ? 'moon' : 'sun'
    const colorSchemeLabel = this.colorScheme === 'dark' ? 'Dark mode' : 'Light mode'

    return html`
      <sl-menu
        class="mt-1 bg-gray-50 border border-gray-200"
        id="preview-menu"
        @sl-select=${this.handleSelect}
      >
        <sl-menu-label>
          <div class="flex items-center gap-2">Appearance</div>
        </sl-menu-label>
        <sl-menu-item @click=${() => this.toggleColorScheme()}>
          <stencila-ui-icon name="${colorSchemeIcon}" slot="prefix"></stencila-ui-icon>
          <span class="text-sm">${colorSchemeLabel}</span>
        </sl-menu-item>
        <sl-menu-label>
          <div class="flex items-center gap-2 ml-4 text-xs text-gray-600">Theme</div>
        </sl-menu-label>
        ${(() => {
          const customTheme = this.getCustomThemeInfo()
          if (customTheme) {
            const iconName = customTheme.type === 'user' ? 'person' : 'gear'
            const displayName = customTheme.name.charAt(0).toUpperCase() + customTheme.name.slice(1)

            return html`
              <sl-menu-item
                type="checkbox"
                ?checked=${this.theme === customTheme.type}
                @click=${() => this.changeTheme(customTheme.type)}
                class="ml-2"
              >
                <stencila-ui-icon name="${iconName}" slot="prefix"></stencila-ui-icon>
                <span class="text-sm">${displayName}</span>
              </sl-menu-item>
            `
          }
          return ''
        })()}
        <sl-menu-item
          type="checkbox"
          ?checked=${this.theme === 'stencila'}
          @click=${() => this.changeTheme('stencila')}
          class="ml-2"
        >
          <stencila-ui-icon name="stencilaColor" slot="prefix"></stencila-ui-icon>
          <span class="text-sm">Stencila</span>
        </sl-menu-item>
        <sl-menu-item
          type="checkbox"
          ?checked=${this.theme === 'tufte'}
          @click=${() => this.changeTheme('tufte')}
          class="ml-2"
        >
          <stencila-ui-icon name="cardText" slot="prefix"></stencila-ui-icon>
          <span class="text-sm">Tufte</span>
        </sl-menu-item>
        <sl-menu-item
          type="checkbox"
          ?checked=${this.theme === 'latex'}
          @click=${() => this.changeTheme('latex')}
          class="ml-2"
        >
          <stencila-ui-icon name="latex" slot="prefix"></stencila-ui-icon>
          <span class="text-sm">LaTeX</span>
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
