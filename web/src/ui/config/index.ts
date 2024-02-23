import { consume } from '@lit/context'
import { Twind, css } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, state } from 'lit/decorators.js'

import { RestAPIClient } from '../../clients/RestAPIClient'
import { SidebarContext, sidebarContext } from '../../contexts/sidebar-context'
import { withTwind } from '../../twind'
import type { Secret } from '../../types/api'

type ICON_KEYS =
  | 'ANTHROPIC_API_KEY'
  | 'GOOGLE_AI_API_KEY'
  | 'OPENAI_API_KEY'
  | 'OLLAMA_API_KEY'
  | 'MISTRAL_API_KEY'

const API_ICONS: Record<ICON_KEYS, string> = {
  ANTHROPIC_API_KEY: 'settings',
  GOOGLE_AI_API_KEY: 'google-LOGO',
  OPENAI_API_KEY: 'open-ai-LOGO',
  OLLAMA_API_KEY: 'ollama-LOGO',
  MISTRAL_API_KEY: 'mistral-LOGO',
}

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
    return html` <div
        class="w-screen h-screen overflow-none fixed top-0 left-0 bg-white z-10 opacity-50"
      ></div>
      <div
        class="flex w-screen min-h-screen fixed top-0 left-0 z-[12] items-center justify-center"
      >
        <div
          class="shadow rounded-md w-full max-w-[528px] min-h-[588px] px-6 pb-6 pt-[18px] inline-flex flex-col justify-start items-start bg-blue-50"
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
                class="ml-auto"
              ></stencila-ui-icon-button>
            </div>
          </header>

          <div class="flex-grow my-[18px] mr-auto">
            ${this.secrets.map((secret) => this.renderSecret(secret))}
          </div>

          <footer class="flex w-full justify-end items-center gap-4">
            <button>discard</button>
            ${this.renderButton()}
          </footer>
        </div>
      </div>`
  }

  private renderButton() {
    const theme = this.tw.theme()
    const buttonDefault = theme.colors['blue-700'] as string
    const buttonHover = theme.colors['blue-800'] as string
    const white = theme.colors['white'] as string
    const fontSize = '14px'

    const styles = css`
      &::part(base) {
        --sl-input-height-medium: 26px;

        border-radius: 3px;
        border-width: 0;
        box-shadow: 0px 1px 0px 0px rgba(255, 255, 255, 0.25) inset;
        background-color: ${buttonDefault};
        color: ${white};
        line-height: 0;
        display: flex;
        flex-direction: row;
        font-size: ${fontSize};
        align-items: center;
        justify-content: center;
        font-weight: 500;
        padding: 6px 36px;

        &:hover {
          background-color: ${buttonHover};
        }
      }

      &::part(label) {
        margin: auto;
        display: contents;
      }
    `

    return html`<sl-button class="${styles}">Save</sl-button>`
  }

  private renderSecret(secret: Secret) {
    const { name, title, description, redacted } = secret
    const icon = API_ICONS[name as ICON_KEYS] ?? ''

    return html`<div
      class="px-6 w-full max-w-[382px] justify-start items-start gap-3 inline-flex"
    >
      <sl-icon library="stencila" name="${icon}" class="text-base"></sl-icon>
      <div
        class="flex-col w-full justify-start items-start gap-1.5 inline-flex"
      >
        <div class="justify-start items-center gap-1.5 inline-flex">
          <div class="text-blue-900 text-xs font-normal">${title}</div>
        </div>
        <div class="opacity-60 text-blue-900 text-xs gap-1.5 font-normal">
          ${description}
        </div>

        ${this.renderInputField(redacted ?? '')}
      </div>
    </div>`
  }

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

  override async firstUpdated() {
    const secrets = await RestAPIClient.listSecrets()

    if (secrets.status === 'success') {
      console.log(secrets.response)
      this.secrets = secrets.response
    }
  }
}
