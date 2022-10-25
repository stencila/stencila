import { html } from 'lit'
import { customElement, state } from 'lit/decorators'
import { apply as twApply, css } from 'twind/css'

import '@shoelace-style/shoelace/dist/components/breadcrumb-item/breadcrumb-item'
import '@shoelace-style/shoelace/dist/components/breadcrumb/breadcrumb'
import '@shoelace-style/shoelace/dist/components/button/button'
import '@shoelace-style/shoelace/dist/components/dialog/dialog'
import '@shoelace-style/shoelace/dist/components/divider/divider'
import '@shoelace-style/shoelace/dist/components/dropdown/dropdown'
import '@shoelace-style/shoelace/dist/components/menu-item/menu-item'
import '@shoelace-style/shoelace/dist/components/menu-label/menu-label'
import '@shoelace-style/shoelace/dist/components/menu/menu'

import { LogLevel } from '../../clients/document-client'
import { DevStatus, devStatusTag } from '../../dev-status'
import {
  currentMode,
  Mode,
  modeDesc,
  modeDevStatus,
  modeFromString,
  modeIcon,
  modeLabel,
} from '../../mode'
import '../base/icon'
import '../base/icon-button'
import { twSheet } from '../utils/css'
import StencilaElement from '../utils/element'
import SlDialog from '@shoelace-style/shoelace/dist/components/dialog/dialog'

const { tw, sheet } = twSheet()

const config = {
  logo: {
    url: 'https://stenci.la/img/stencila/stencilaLogo.svg',
    alt: 'Stencila',
  },
  //title: 'Docs',
  /*
  links: [
    { label: 'Tutorials', url: '#tutorials', icon: 'stars' },
    { label: 'Guides', url: '#guides', icon: 'map' },
    { label: 'Reference', url: '#reference', icon: 'book' },
  ],
  */
  modes: [
    'Static',
    'Dynamic',
    'Interact',
    'Inspect',
    'Alter',
    'Develop',
    'Edit',
    'Write',
    'Code',
    'Shell',
  ],
  /*
  breadcrumbs: [
    { label: 'One', path: 'one' },
    { label: 'Two', path: 'two' },
    { label: 'Three', path: 'three' },
    { label: 'Four' },
  ],
  */
}
type Config = typeof config

/**
 * A component for a document header
 */
@customElement('stencila-document-header')
export default class StencilaDocumentHeader extends StencilaElement {
  static styles = [sheet.target]

  constructor() {
    super()

    // Request update of this component on client connection / disconnection events
    const requestUpdate = (event: Event) => this.requestUpdate()
    window.addEventListener('stencila-client-connecting', requestUpdate)
    window.addEventListener('stencila-client-connected', requestUpdate)
    window.addEventListener('stencila-client-disconnected', requestUpdate)
  }

  protected render() {
    return html`<header
      class=${tw(css`
        ${twApply('font-sans')}

        /* Reduce contrast of hovered menu item background */
        sl-menu-item::part(base):hover {
          background-color: var(--sl-color-primary-50);
        }
      `)}
    >
      ${this.renderTopbar(config)}
      ${config.breadcrumbs?.length > 1
        ? this.renderBreadcrumbs(config.breadcrumbs)
        : ''}
      ${this.renderKeyboardDialog()}
    </header>`
  }

  private renderTopbar(config: Config) {
    const { logo, title, links, modes } = config
    return html`<nav
      class="${tw`sm:fixed top-0 left-0 right-0 z-50 border(b gray-200) bg-white`}"
    >
      <div class="${tw`mx-auto max-w-7xl px-4 sm:px-6 lg:px-8`}">
        <div class="${tw`flex h-16 justify-between`}">
          <div class="${tw`flex`}">
            ${logo && this.renderLogo(logo)} ${title && this.renderTitle(title)}
            ${links && this.renderLinks(links)}
          </div>
          <div class="${tw`flex`}">
            ${this.renderModeMenu(modes)} ${this.renderDesktopMenu(modes)}
            ${this.renderMobileMenu(links, modes)}
          </div>
        </div>
      </div>
    </nav>`
  }

  private renderBreadcrumbs(breadcrumbs: { label: string; path?: string }[]) {
    return html` <div class="${tw`border(b gray-200) bg-white h-8 pt-1.5`}">
      <div
        class="${tw(css`
          ${twApply('mx-auto max-w-7xl px-4 sm:px-6 lg:px-8')}
          sl-breadcrumb-item::part(base) {
            ${twApply('text(gray-700 sm) font(sans normal)')}
          }
        `)}"
      >
        <sl-breadcrumb>
          <sl-breadcrumb-item>
            <a href="/"
              ><stencila-icon slot="prefix" name="house"></stencila-icon
            ></a>
          </sl-breadcrumb-item>
          ${breadcrumbs.map(
            ({ label, path }, index) => html`<sl-breadcrumb-item>
              ${index < breadcrumbs.length - 1
                ? html`<a href="${path}" class="${tw`text-blue-700`}"
                    >${label}</a
                  >`
                : label}
            </sl-breadcrumb-item>`
          )}
        </sl-breadcrumb>
      </div>
    </div>`
  }

  private renderLogo({ url, alt = 'Logo' }: { url: string; alt: string }) {
    return html`<div class="${tw`flex flex-shrink-0 items-center`}">
      <a href="/"
        ><img class="${tw`block h-8 w-auto`}" src="${url}" alt="${alt}"
      /></a>
    </div>`
  }

  private renderTitle(title: string) {
    return html`<div
      class="${tw`flex items-center ml-3 mr-5 text(gray-700 lg) font-medium`}"
    >
      ${title}
    </div>`
  }

  private renderLinks(links: { label: string; url: string; icon?: string }[]) {
    return html`<div class="${tw`hidden sm:(-my-px ml-6 flex space-x-8)`}">
      ${links.map(({ label, url, icon }) => {
        const active =
          new URL(url, window.location.href).href == window.location.href
        const aria = active ? 'aria-current="page"' : ''
        const cls = active
          ? tw`border(blue-500 b-2) text(gray-800 sm) inline-flex items-center px-1 pt-1`
          : tw`border(transparent b-2) text(gray-500 sm) hover:(border-gray-300 text-gray-700) inline-flex items-center px-1 pt-1`
        return html`<a href="${url}" ${aria} class="${cls}"
          >${icon &&
          html`<stencila-icon
            name="${icon}"
            class="${tw`mr-1`}"
          ></stencila-icon>`}
          <span>${label}</span>
        </a>`
      })}
    </div>`
  }

  private renderModeMenu(modes?: string[]) {
    const mode = currentMode()
    return html`
      <div
        class="${tw(css`
          ${twApply('flex items-center invisible sm:visible')}
          sl-button::part(base) {
            ${twApply('border(none) pt-3 text(gray-500 xl)')}
          }
          sl-dropdown::part(panel) {
            ${twApply('border(1 solid gray-100) rounded')}
          }
          sl-menu-label::part(base) {
            ${twApply('text(gray-400 xs)')}
          }
        `)}"
      >
        <sl-dropdown>
          <sl-button slot="trigger" variant="default" size="large" circle
            ><stencila-icon name="${modeIcon(mode)}"></stencila-icon
          ></sl-button>
          <sl-menu>
            ${modes && modes.length > 1 && this.renderModeMenuItems(modes)}
          </sl-menu>
        </sl-dropdown>
      </div>
    `
  }

  private renderModeMenuItems(modes: string[]) {
    const currMode = currentMode()
    return modes.map((modeName) => {
      const mode = modeFromString(modeName)
      const label = modeLabel(mode)
      const devStatus = modeDevStatus(mode)
      const target = mode > Mode.Write ? '_blank' : ''
      const disabled = devStatus < DevStatus.Alpha
      return html`<a
        href="${disabled ? '#' : `?mode=${label.toLowerCase()}`}"
        target="${target}"
        class="${tw(css`
          sl-menu-item::part(base) {
            ${twApply(
              `text(gray-${disabled ? 400 : 600} sm) font-medium mt-1 cursor-${
                disabled ? 'default' : 'pointer'
              }`
            )}
          }
          stencila-tag {
            ${twApply('float-right')}
          }
          .label {
            ${twApply(`ml-1`)}
          }
          .desc {
            ${twApply(`text(gray-400 xs) font-light`)}
          }
        `)}"
        ><sl-menu-item
          class="${mode == currMode ? tw`border(blue-500 l-2)` : ''}"
        >
          <div>
            <stencila-icon
              slot="prefix"
              name="${modeIcon(mode)}"
            ></stencila-icon>
            <span class="label">${label}</span>
            ${devStatus != DevStatus.Stable
              ? devStatusTag(devStatus, 'xxs')
              : ''}
          </div>
          <div class="desc">${modeDesc(mode)}</div>
        </sl-menu-item></a
      >`
    })
  }

  private renderDesktopMenu(modes?: string[]) {
    return html`
      <div
        class="${tw(css`
          ${twApply('flex items-center invisible sm:visible')}
          sl-button::part(base) {
            ${twApply('border(none) pt-3 text(gray-500 xl)')}
          }
          sl-dropdown::part(panel) {
            ${twApply('border(1 solid gray-100) rounded')}
          }
          sl-menu-label::part(base) {
            ${twApply('text(gray-400 xs)')}
          }
          sl-menu-item::part(base) {
            ${twApply('text(gray-600 sm) font-medium mt-1')}
          }
          stencila-tag {
            ${twApply('float-right')}
          }
          .title {
            ${twApply('')}
          }
          .label {
            ${twApply(`ml-1`)}
          }
          .desc {
            ${twApply('text(gray-400 xs) font-light')}
          }
        `)}"
      >
        <sl-dropdown>
          <sl-button slot="trigger" variant="default" size="large" circle
            ><stencila-icon name="three-dots-vertical"></stencila-icon
          ></sl-button>
          <sl-menu>
            ${this.renderConnectionMenuItem()} ${this.renderKeyboardMenuItem()}
            ${this.renderDebugMenuItem()}
          </sl-menu>
        </sl-dropdown>
      </div>
    `
  }

  renderConnectionMenuItem() {
    const mode = currentMode()
    const client = window.stencilaClient
    if (mode == Mode.Static || !client) {
      return html``
    }

    const status = client.status()
    if (
      status === 'connecting' ||
      status === 'connected' ||
      status === 'reconnecting'
    ) {
      return html`<sl-menu-item
        @click=${() => window.stencilaClient.disconnect()}
      >
        <div>
          <stencila-icon name="wifi-off"></stencila-icon>
          Disconnect
        </div>
        <div class="${tw`text-xs font-light`}">
          Currently ${status} to server
        </div>
      </sl-menu-item>`
    } else {
      return html`<sl-menu-item @click=${() => window.stencilaClient.connect()}>
        <div>
          <stencila-icon name="wifi"></stencila-icon>
          Connect
        </div>
        <div class="${tw`text-xs font-light`}">
          Currently disconnected from server
        </div>
      </sl-menu-item>`
    }
  }

  renderDebugMenuItem() {
    const on =
      (window.stencilaClient?.logLevel ?? LogLevel.Info) == LogLevel.Debug
    return html`<sl-menu-item
      @click=${() => {
        window.stencilaClient.changeLogLevel(
          on ? LogLevel.Info : LogLevel.Debug
        )
        this.requestUpdate()
      }}
    >
      <div>
        <stencila-icon name="bug"></stencila-icon>
        Turn ${on ? 'off' : 'on'} debugging
      </div>
      <div class="${tw`text-xs font-light`}">
        Debug level logging is currently ${on ? 'enabled' : 'disabled'}
      </div>
    </sl-menu-item>`
  }

  renderKeyboardMenuItem() {
    return html`<sl-menu-item
      @click=${() => {
        const dialog = this.renderRoot.querySelector(
          '#keyboard-dialog'
        )! as SlDialog
        return dialog.show()
      }}
    >
      <div>
        <stencila-icon name="keyboard"></stencila-icon>
        Keyboard shortcuts
      </div>
      <div class="${tw`text-xs font-light`}">
        Get help on which keyboard shortcuts to use where
      </div>
    </sl-menu-item>`
  }

  renderKeyboardDialog() {
    return html`<sl-dialog
      label="Keyboard shortcuts"
      id="keyboard-dialog"
      class=${tw(css`
        h2 {
          ${twApply('mb-3 text(lg gray-600)')}
        }
        p {
          ${twApply('mb-2 text(sm gray-500)')}
        }
        kbd {
          ${twApply('mr-2 rounded border(1 gray-400) p-1 text(xs gray-400)')}
        }
      `)}
    >
      <h2>
        <stencila-icon name="code"></stencila-icon>
        Code editors
      </h2>
      <dl>
        <p><kbd>Tab</kbd>Indent line/s</p>
        <p><kbd>Esc+Tab</kbd>Do not indent, move to next input</p>
        <p><kbd>Esc+Shift+Tab</kbd>Do not indent, move to previous input</p>
        <p><kbd>Ctrl+Space</kbd>Bring up autocompletion prompt</p>
        <p><kbd>Ctrl+Enter</kbd>Execute the code (if executable)</p>
      </dl>
    </sl-dialog>`
  }

  @state()
  private mobileMenuIsOpen: boolean = false

  private renderMobileMenu(
    links?: { label: string; url: string; icon?: string }[],
    modes?: string[]
  ) {
    return html`
      <div
        class="${tw(css`
          ${twApply(`flex items-center sm:hidden`)}
          stencila-icon-button {
            ${twApply('text(gray-700 xl) font-medium')}
          }
          sl-dropdown::part(panel) {
            ${twApply('border(1 solid gray-100) rounded')}
          }
          sl-menu-label::part(base) {
            ${twApply('text(gray-400 xs)')}
          }
          sl-menu-item::part(base) {
            ${twApply('text(gray-600 sm) font-medium mt-1')}
          }
          stencila-tag {
            ${twApply('float-right')}
          }
          .title {
            ${twApply('')}
          }
          .label {
            ${twApply(`ml-1`)}
          }
          .desc {
            ${twApply('text(gray-400 xs) font-light')}
          }
        `)}"
      >
        <sl-dropdown
          @sl-show=${() => (this.mobileMenuIsOpen = true)}
          @sl-hide=${() => (this.mobileMenuIsOpen = false)}
        >
          <stencila-icon-button
            slot="trigger"
            name="${this.mobileMenuIsOpen ? 'x-lg' : 'list'}"
          ></stencila-icon-button>
          <sl-menu>
            ${links &&
            links.length > 1 &&
            links.map(
              ({ url, label, icon }) =>
                html`<a href="${url}"
                  ><sl-menu-item
                    >${icon &&
                    html`<stencila-icon
                      slot="prefix"
                      name="${icon}"
                    ></stencila-icon>`}${label}</sl-menu-item
                  ></a
                >`
            )}
            ${modes &&
            modes.length > 1 &&
            html`<sl-divider></sl-divider>
              <sl-menu-label>Mode</sl-menu-label>
              ${this.renderModeMenuItems(modes)}`}
            <sl-divider></sl-divider>
            ${this.renderConnectionMenuItem()} ${this.renderDebugMenuItem()}
          </sl-menu>
        </sl-dropdown>
      </div>
    `
  }
}
