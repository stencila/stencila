import { consume } from '@lit/context'
import { Twind, css } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, state } from 'lit/decorators.js'

import { RestAPIClient } from '../../clients/RestAPIClient'
import { SidebarContext, sidebarContext } from '../../contexts/sidebar-context'
import { emitSidebarEvent } from '../../events/sidebar'
import { withTwind } from '../../twind'
import type { Secret } from '../../types/api'

import { API_ICONS, ICON_KEYS } from './icons'

/**
 * UI config screen
 *
 * Displays a number of settings that users can change in the app as needed.
 */
@customElement('stencila-ui-config-screen')
@withTwind()
export class ConfigScreen extends LitElement {
  @consume({ context: sidebarContext, subscribe: true })
  context: SidebarContext

  // Set the type on the `tw` var
  private tw: Twind

  @state()
  protected secrets: Secret[] = []

  override render() {
    return html`${this.renderOverlay()} ${this.renderConfigPanel()}`
  }

  /**
   * Render the panel section - which sites on top of the overlay.
   */
  private renderConfigPanel() {
    return html`<div
      class="${this.context.configOpen
        ? 'top-1/2 -translate-y-1/2'
        : 'top-full'} transition-all duration-300 fixed left-1/2 -translate-x-1/2 z-[12] w-full max-w-[528px] min-h-[588px]"
    >
      <div
        class="shadow rounded-md m-5 px-6 pb-6 pt-[18px] inline-flex flex-col justify-start items-start bg-blue-50"
      >
        <header
          class="self-stretch h-9 flex-row justify-start items-start gap-3 flex border-b-2 border-blue-200"
        >
          <div
            class="grow shrink basis-0 text-blue-900 text-base flex items-center gap-3"
          >
            <sl-icon
              library="stencila"
              name="settings"
              class="fill-blue-900"
            ></sl-icon>
            Settings
          </div>
          <div class="grow shrink basis-0 text-base flex items-end gap-3 h-6">
            <stencila-ui-icon-button
              icon="close-button"
              size="14px"
              class="ml-auto fill-blue-400 hover:fill-blue-800"
              .clickEvent=${this.handleClose}
              ?ignoreColours=${true}
            ></stencila-ui-icon-button>
          </div>
        </header>

        <div class="flex-grow my-[18px] mr-auto">
          ${this.secrets.map((secret) => this.renderSecret(secret))}
        </div>

        <footer class="flex w-full justify-end items-center gap-4">
          <button>discard</button>
          <stencila-ui-button>Save me</stencila-ui-button>
        </footer>
      </div>
    </div>`
  }

  /**
   * Renders the background overlay for the component.
   */
  private renderOverlay() {
    return html`<div
      class="transition w-screen h-screen overflow-none fixed top-0 left-0  z-10 bg-white ${this
        .context.configOpen
        ? 'opacity-50 pointer-events-all cursor-pointer'
        : 'opacity-0 pointer-events-none'}"
      @click=${this.handleClose}
    ></div>`
  }

  /**
   * Renders an individual secret.
   */
  private renderSecret(secret: Secret) {
    const { name, title, description, redacted } = secret
    const icon = API_ICONS[name as ICON_KEYS] ?? ''

    return html`<div
      class="px-6 w-full max-w-[382px] justify-start items-start gap-3 inline-flex"
    >
      <sl-icon
        library="stencila"
        name="${icon}"
        class="text-2xl opacity-70 flex-shrink-0 flex-grow-0"
      ></sl-icon>
      <div class="flex-col w-full justify-start items-start inline-flex">
        <div class="text-blue-900 text-xs font-normal leading-relaxed">
          ${title}
        </div>
        <div class="opacity-60 text-blue-900 text-xs font-normal mb-2">
          ${description}
        </div>

        ${this.renderInputField(redacted ?? '')}
      </div>
    </div>`
  }

  /**
   * Render an individual input field (as seen in the list of secrets).
   */
  private renderInputField(value: string) {
    const styles = css`
      &::part(form-control) {
        --sl-input-border-radius-small: 3px;
        --sl-input-font-size-small: 10px;
        --sl-input-color: #999999;
        --sl-input-border-color: none;
        --sl-input-border-width: 0;
        --sl-input-border-color-focus: transparent;
        --sl-focus-ring-width: 1px;
        --sl-input-focus-ring-color: #092d77;
        --sl-input-height-small: 20px;
      }

      &::part(form-control) {
        width: 100%;
      }

      &::part(input) {
        --sl-input-spacing-small: 8px;
        padding: 4px var(--sl-input-spacing-small);
        box-shadow: 0px 0px 4px 0px rgba(0, 0, 0, 0.25) inset;
      }

      &::part(form-control-help-text) {
        padding: 0 var(--sl-input-spacing-small);
        color: var(--sl-input-focus-ring-color);
      }
    `

    return html`
      <div class="mb-5 w-full">
        <sl-input
          class="${styles} w-full"
          size="small"
          value=${value}
          help-text="&nbsp;"
        ></sl-input>
      </div>
    `
  }

  /**
   * Handle the "close" event - to hide the config panel. Updates the sidebar
   * context's `configOpen` value.
   */
  private handleClose = () => {
    const event = emitSidebarEvent('stencila-config-toggle', {
      configOpen: false,
    })
    this.dispatchEvent(event)
  }

  override async firstUpdated() {
    const secrets = await RestAPIClient.listSecrets()

    if (secrets.status === 'success') {
      this.secrets = secrets.response
    }
  }
}
