import { consume } from '@lit/context'
import { LitElement, html } from 'lit'
import { customElement, state } from 'lit/decorators.js'

import { RestAPIClient } from '../../clients/RestAPIClient'
import { SidebarContext, sidebarContext } from '../../contexts/sidebar-context'
import { emitSidebarEvent } from '../../events/sidebar'
import { withTwind } from '../../twind'

import { API_ICONS, ICON_KEYS } from './icons'
import { SavedState, SecretName, SecretState } from './types'

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

  /**
   * Store all secrets found in the API & all modifications to them.
   */
  @state()
  private secrets: {
    [Property in SecretName]?: SecretState
  }

  /**
   * The state as it changes when
   */
  @state()
  private savedState: SavedState = 'idle'

  override render() {
    return html`<stencila-ui-overlay
        .isOpen=${this.context.configOpen}
        .handleClose=${this.handleClose}
      ></stencila-ui-overlay>
      ${this.renderConfigPanel()}`
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
          ${this.getSecretsKeys().map((secret) => {
            if (this.secrets[secret]) {
              return this.renderSecret(this.secrets[secret])
            }
            return html``
          })}
        </div>

        <footer class="flex w-full justify-end">
          <stencila-ui-button
            theme="blue-inline-text--small"
            class="inline-block h-auto"
            .clickEvent=${() => {
              this.handleClose()
            }}
            >Discard</stencila-ui-button
          >
          <stencila-ui-button
            theme="blue-solid"
            class="inline-block h-auto"
            .clickEvent=${() => {
              this.handleSave()
            }}
            .disabled=${this.savedState === 'saving'}
            >Save me</stencila-ui-button
          >
        </footer>
      </div>
    </div>`
  }

  /**
   * Helper to correctly type the secrets we get back from the API.
   */
  private getSecretsKeys() {
    if (!this.secrets) {
      return [] as SecretName[]
    }

    return Object.keys(this.secrets) as SecretName[]
  }

  /**
   * Renders an individual secret.
   */
  private renderSecret(secret: SecretState) {
    const { name, title, description, redacted } = secret.original
    const icon = API_ICONS[name as ICON_KEYS] ?? ''
    const inputValue =
      (this.context.configOpen ? secret.modifiedValue : undefined) ??
      redacted ??
      ''

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

        <stencila-ui-input-field
          defaultValue=${redacted ?? ''}
          value=${inputValue}
          .isConfigOpen=${this.context.configOpen ?? false}
          .changeEvent=${this.handleInputChangeEvent(secret)}
        ></stencila-ui-input-field>
      </div>
    </div>`
  }

  /**
   * Handles changes to an input field:
   * - updates the state for the specific secret
   * - keeps track of any modifications to the secret
   */
  private handleInputChangeEvent(secret: SecretState) {
    return (element: HTMLInputElement) => {
      secret.modifiedValue = element.value
    }
  }

  /**
   * Find the secrets that have been modified.
   */
  private filterModifiedSecrets() {
    return this.getSecretsKeys()
      .filter((key) => {
        return this.secrets[key].modifiedValue
      })
      .map((key) => {
        return this.secrets[key]
      })
  }

  /**
   * Handle the "close" event - to hide the config panel. Updates the sidebar
   * context's `configOpen` value.
   */
  private handleClose = () => {
    this.resetModifiedSecrets()
    const event = emitSidebarEvent('stencila-config-toggle', {
      configOpen: false,
    })
    this.dispatchEvent(event)
  }

  private async handleSave() {
    const toUpdate = this.filterModifiedSecrets()

    this.savedState = 'saving'

    const savedAPIs = Promise.allSettled(
      toUpdate.map((secret) => {
        return RestAPIClient.setSecret(
          secret.original.name,
          secret.modifiedValue
        )
      })
    )

    const results = await savedAPIs
    const hasError = results.some((result) => {
      return (
        result.status === 'rejected' ||
        (result.status === 'fulfilled' && result.value.status === 'error')
      )
    })

    // get the secrets form the server.
    // - ensures we get all updates to secrets
    // - resets modified values
    const getSecrets = await this.getSecrets()

    this.savedState = hasError || !getSecrets ? 'error' : 'done'
  }

  private resetModifiedSecrets() {
    const toMutate = {
      ...this.secrets,
    }
    for (const secret of this.filterModifiedSecrets()) {
      secret.modifiedValue = undefined
      toMutate[secret.original.name as SecretName] = { ...secret }
    }

    this.secrets = { ...toMutate }
    this.requestUpdate()
  }

  /**
   * Retrieve the secrets from the API.
   */
  private async getSecrets() {
    try {
      const secrets = await RestAPIClient.listSecrets()

      if (secrets.status === 'error') {
        return false
      }

      this.secrets = secrets.response.reduce<typeof this.secrets>(
        (acc, secret) => {
          acc[secret.name as SecretName] = {
            original: secret,
          }
          return acc
        },
        {}
      )

      return true
    } catch {
      return false
    }
  }

  override async firstUpdated() {
    await this.getSecrets()
  }
}
