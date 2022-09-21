import { html } from 'lit'
import { customElement, state } from 'lit/decorators'
import { apply as twApply, css } from 'twind/css'

import '@shoelace-style/shoelace/dist/components/breadcrumb/breadcrumb'
import '@shoelace-style/shoelace/dist/components/breadcrumb-item/breadcrumb-item'
import '@shoelace-style/shoelace/dist/components/button/button'
import '@shoelace-style/shoelace/dist/components/divider/divider'
import '@shoelace-style/shoelace/dist/components/dropdown/dropdown'
import '@shoelace-style/shoelace/dist/components/menu-item/menu-item'
import '@shoelace-style/shoelace/dist/components/menu-label/menu-label'
import '@shoelace-style/shoelace/dist/components/menu/menu'

import { Mode, modeDesc, modeFromString, modeToString } from '../../mode'
import StencilaElement from '../base/element'
import { IconName } from '../base/icon'
import '../base/icon-button'
import { twSheet } from '../utils/css'

const { tw, sheet } = twSheet()

const currentMode = Mode.Interact
const config = {
  logo: {
    url: 'https://stenci.la/img/stencila/stencilaLogo.svg',
    alt: 'Stencila',
  },
  title: 'Docs',
  links: [
    { label: 'Tutorials', url: '#tutorials', icon: 'stars' },
    { label: 'Guides', url: '#guides', icon: 'map' },
    { label: 'Reference', url: '#reference', icon: 'book' },
  ],
  views: [
    'Static',
    'Read',
    'Interact',
    'Inspect',
    'Alter',
    'Develop',
    'Edit',
    'Write',
  ],
  breadcrumbs: [
    { label: 'One', path: 'one' },
    { label: 'Two', path: 'two' },
    { label: 'Three', path: 'three' },
    { label: 'Four' },
  ],
}

/**
 * A component for a document header
 */
@customElement('stencila-document-header')
export default class StencilaDocumentHeader extends StencilaElement {
  static styles = [sheet.target]

  render() {
    const { logo, title, links, views, breadcrumbs } = config
    return html`<nav class="${tw`border(b gray-200) bg-white font-sans`}">
        <div class="${tw`mx-auto max-w-7xl px-4 sm:px-6 lg:px-8`}">
          <div class="${tw`flex h-16 justify-between`}">
            <div class="${tw`flex`}">
              ${logo && this.renderLogo(logo)}
              ${title && this.renderTitle(title)}
              ${links && this.renderLinks(links)}
            </div>
            ${this.renderDesktopMenu(views)}
            ${this.renderMobileMenu(links, views)}
          </div>
        </div>
      </nav>
      ${breadcrumbs?.length > 1 && this.renderBreadcrumbs(breadcrumbs)}`
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

  private renderDesktopMenu(views: string[]) {
    return html`
      <div
        class="${tw(css`
          ${twApply('flex items-center invisible sm:visible')}
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
        `)}"
      >
        <sl-dropdown>
          <stencila-icon-button
            slot="trigger"
            name="three-dots-vertical"
          ></stencila-icon-button>
          <sl-menu>
            ${views?.length > 1 &&
            html`<sl-divider></sl-divider>
              <sl-menu-label>Views</sl-menu-label>
              ${this.renderViewMenuItems(views)}`}
          </sl-menu>
        </sl-dropdown>
      </div>
    `
  }

  @state()
  private mobileMenuIsOpen: boolean = false

  private renderMobileMenu(
    links: { label: string; url: string; icon?: string }[],
    views: string[]
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
            ${links.map(
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
            ${views?.length > 1 &&
            html`<sl-divider></sl-divider>
              <sl-menu-label>Views</sl-menu-label>
              ${this.renderViewMenuItems(views)}`}
          </sl-menu>
        </sl-dropdown>
      </div>
    `
  }

  private renderViewMenuItems(views: string[]) {
    return views.map((view) => {
      const mode = modeFromString(view)
      const modeString = modeToString(mode)
      const active = mode == currentMode
      const cls = active ? tw`border(blue-500 l-2) bg-blue-50` : ''
      return html`<a href="?mode=${modeString.toLowerCase()}"
        ><sl-menu-item class="${cls}">
          <div>
            <stencila-icon
              slot="prefix"
              name="${this.modeIcon(mode)}"
            ></stencila-icon>
            <span class=${tw`ml-1`}>${modeString}</span>
          </div>
          <div class=${tw`text(gray-400 xs) font-light`}>${modeDesc(mode)}</div>
        </sl-menu-item></a
      >`
    })
  }

  private modeIcon(mode: Mode): IconName {
    switch (mode) {
      case Mode.Static:
        return 'book'
      case Mode.Read:
        return 'wifi'
      case Mode.Inspect:
        return 'search'
      case Mode.Interact:
        return 'sliders'
      case Mode.Alter:
        return 'play'
      case Mode.Develop:
        return 'code'
      case Mode.Edit:
        return 'pencil'
      case Mode.Write:
        return 'pen'
    }
  }
}
