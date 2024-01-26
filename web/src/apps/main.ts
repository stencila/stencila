import { html } from 'lit'
import { customElement, property } from 'lit/decorators.js'

// import logo from '../images/stencilaIcon.svg'
import { THEMES } from '../themes/themes'
import type { DocumentId, DocumentView } from '../types'
import type { UISelectorSelectedEvent } from '../ui/selector'
import '../ui/selector'
import { TWLitElement } from '../ui/twind'
import { VIEWS } from '../views/views'

import '../views/static'
import '../views/live'
import '../views/dynamic'
import '../views/source'
import '../views/split'
import '../views/visual'

import './main.css'

import './shoelace'

/**
 * Application Wrapper
 *
 * Wraps the application in the `app-chrome`. Contains the main header and
 * footer.
 */
@customElement('stencila-main-app')
export class App extends TWLitElement {
  /**
   * The id of the current document (if any)
   *
   * The app can be opened with a document or not. If there is no `doc` attribute
   * (e.g. because the server could not resolve a file from the URL path)
   * then the app should offer some suggestions.
   */
  @property()
  doc?: DocumentId

  /**
   * The current view of the current document
   *
   * If there is no `view` attribute then this will be dynamically
   * determined based on the maximum access level that the user has for
   * the document.
   */
  @property()
  view?: DocumentView = 'live'

  /**
   * The theme to use for HTML-based views of the document (e.g. `static`, `live`)
   */
  @property()
  theme: string = 'default'

  /**
   * The format to use for source views of the document (`source` and `split` view)
   */
  @property()
  format: string = 'markdown'

  override render() {
    return html`<div
      class="font-sans flex flex-row bg-neutral-100 min-h-screen w-full"
    >
      <div>sidebar</div>
      <div class="flex flex-col flex-grow">
        ${this.renderHeader()}
        <div class="bg-white h-full">editor</div>
      </div>
    </div>`
  }

  private renderHeader() {
    return html`<header class="w-full flex items-end h-20">
      <nav class="flex bg-red-400 h-full w-full">
        <div class="flex-grow flex items-end h-full">tabs</div>
        <div class="flex-shrink-0 flex items-end">
          <div
            class="flex-grow justify-start flex flex-col sm:flex-row sm:space-x-4"
          >
            ${this.renderViewSelect()} ${this.renderThemeSelect()}
          </div>
        </div>
      </nav>
    </header>`
  }

  private renderViewSelect() {
    const clickEvent = (e: UISelectorSelectedEvent['detail']) => {
      console.log(e)
      this.view = e.item.value as DocumentView
    }

    return html` <stencila-ui-selector
      label="View"
      target=${this.view}
      targetClass="view-selector"
      .list=${Object.entries(VIEWS)}
      .clickEvent=${clickEvent}
    >
    </stencila-ui-selector>`
  }

  private renderThemeSelect() {
    const clickEvent = (e: UISelectorSelectedEvent['detail']) => {
      this.theme = e.item.value
    }

    return html` <stencila-ui-selector
      label="Theme"
      target=${this.theme}
      targetClass="theme-selector"
      .list=${Object.entries(THEMES)}
      .clickEvent=${clickEvent}
    >
    </stencila-ui-selector>`
  }

  /* eslint-disable lit/attribute-value-entities */
  private renderPrintLink() {
    return html`<a
      href="?mode=doc&view=print&theme=${this.theme}"
      target="_blank"
      >Print preview</a
    >`
  }
  /* eslint-enable lit/attribute-value-entities */

  private renderFooter() {
    return html`<footer class="bg-brand-blue px-4 py-6 text-white">
      <div class="container mx-auto">
        <p class="text-sm my-0">&copy; 2023 Stencila Ltd.</p>
      </div>
    </footer>`
  }

  private renderView() {
    switch (this.view) {
      case 'static':
        return html`<stencila-static-view
          view="static"
          doc=${this.doc}
          theme=${this.theme}
          fetch
        ></stencila-static-view>`

      case 'live':
        return html`<stencila-live-view
          view="live"
          doc=${this.doc}
          theme=${this.theme}
        ></stencila-live-view>`

      case 'dynamic':
        return html`<stencila-dynamic-view
          view="dynamic"
          doc=${this.doc}
          theme=${this.theme}
        ></stencila-dynamic-view>`

      case 'source':
        return html`<stencila-source-view
          view="source"
          doc=${this.doc}
          format=${this.format}
        ></stencila-source-view>`

      case 'split':
        return html`<stencila-split-view
          view="split"
          doc=${this.doc}
          format=${this.format}
          theme=${this.theme}
        ></stencila-split-view>`

      case 'visual':
        return html`<stencila-visual-view
          view="visual"
          doc=${this.doc}
          theme=${this.theme}
        ></stencila-visual-view>`
    }
  }
}
