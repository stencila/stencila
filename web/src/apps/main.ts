import { provide } from '@lit/context'
import type SlTab from '@shoelace-style/shoelace/dist/components/tab/tab'
import '@shoelace-style/shoelace/dist/components/tab/tab'
import '@shoelace-style/shoelace/dist/components/tab-group/tab-group'
import '@shoelace-style/shoelace/dist/components/tab-panel/tab-panel'
import type { SlCloseEvent } from '@shoelace-style/shoelace/dist/events/events'
import type { File } from '@stencila/types'
import { apply, css } from '@twind/core'
import { LitElement, html } from 'lit'
import { customElement, property, state } from 'lit/decorators.js'

import { InfoViewContext, infoviewContext } from '../contexts/infoview-context'
import { SidebarContext, sidebarContext } from '../contexts/sidebar-context'
import { THEMES } from '../themes/themes'
import { withTwind } from '../twind'
import type { DocumentId, DocumentView } from '../types'
import type { UISelectorSelectedEvent } from '../ui/selector'
import '../ui/directory-container'
import '../ui/selector'
import '../ui/sidebar'
import '../ui/tab'
import '../ui/view-container'
import '../ui/buttons/icon'
import '../views/static'
import '../views/live'
import '../views/dynamic'
import '../views/source'
import '../views/split'
import '../views/visual'
import { DirectoryView } from '../views/directory'
import { VIEWS } from '../views/views'

import './main.css'

import './shoelace'

/**
 * Application Wrapper
 *
 * Wraps the application in the `app-chrome`. Contains the main header and
 * footer.
 */
@customElement('stencila-main-app')
@withTwind()
export class App extends LitElement {
  /**
   * The currently open documents
   *
   * This property is initialized (as an HTML attribute) with one document id,
   * by the server, based on the URL path (including paths that resolved to main.*,
   * index.*, or README.* in the home directory of the server (the directory it was started in)).
   *
   * While the app is running [document id, file name] pairs are added or removed
   * from this list (e.g. by clicking on the directory tree, closing a tab).
   *
   * A list is used here (rather than say an object with `DocumentId` as the key)
   * to allow for reordering of tabs by the user.
   */
  @property({ type: Array })
  docs: (File & { docId: DocumentId })[] = []

  /**
   * The id of the document, in `docs`, that is currently active
   */
  @state()
  activeDoc: DocumentId | null

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

  /**
   * This context enables components to:
   * - open the files viewer
   * - change the view by clicking on a sidebar button
   */
  @provide({ context: sidebarContext })
  @state()
  contextObject: SidebarContext

  /**
   * This context enables opening and closing the
   * info view panel
   */
  @provide({ context: infoviewContext })
  @state()
  infoViewContext: InfoViewContext

  override render() {
    return html`<div
      class="font-sans flex flex-row bg-neutral-100 fixed top-0 left-0 min-h-screen w-full"
    >
      <stencila-ui-sidebar></stencila-ui-sidebar>
      <stencila-ui-directory-container></stencila-ui-directory-container>

      <div class="flex flex-col flex-grow">
        ${this.renderHeader()} ${this.renderTabGroup()}
      </div>
    </div> `
  }

  // TODO: the header should move to it's own component & maintain its own state.
  private renderHeader() {
    const infoClickEvent = () => {
      this.infoViewContext = {
        ...this.infoViewContext,
        infoViewOpen: !this.infoViewContext.infoViewOpen,
      }
    }

    return html`<header class="w-full flex items-end h-12 max-h-12">
      <nav class="flex justify-end bg-neutral-100 h-full w-full">
        <div class="flex-shrink-0 flex-grow-0 flex items-center p-4">
          <div class="ml-20 flex space-x-4">
            <stencila-ui-icon-button
              icon="status"
              ?disabled=${true}
            ></stencila-ui-icon-button>
            <stencila-ui-icon-button
              icon="info"
              .clickEvent=${() => infoClickEvent()}
              ?active=${this.infoViewContext.infoViewOpen}
            ></stencila-ui-icon-button>
            <stencila-ui-icon-button icon="print"></stencila-ui-icon-button>
          </div>
        </div>
      </nav>
    </header>`
  }

  // @ts-expect-error "will use soon enough"
  private renderViewSelect() {
    const clickEvent = (e: UISelectorSelectedEvent['detail']) => {
      this.contextObject = {
        ...this.contextObject,
        currentView: e.item.value as Exclude<DocumentView, 'directory'>,
      }
    }

    return html`<stencila-ui-selector
      label="View"
      target=${this.contextObject.currentView}
      targetClass="view-selector"
      .list=${Object.entries(VIEWS)}
      .clickEvent=${clickEvent}
    >
    </stencila-ui-selector>`
  }

  // @ts-expect-error "will use soon enough"
  private renderThemeSelect() {
    const clickEvent = (e: UISelectorSelectedEvent['detail']) => {
      this.theme = e.item.value
    }

    return html`<stencila-ui-selector
      label="Theme"
      target=${this.theme}
      targetClass="theme-selector"
      .list=${Object.entries(THEMES)}
      .clickEvent=${clickEvent}
    >
    </stencila-ui-selector>`
  }

  /* eslint-disable lit/attribute-value-entities */
  // @ts-expect-error "will use soon enough"
  private renderPrintLink() {
    return html`<a
      href="?mode=doc&view=print&theme=${this.theme}"
      target="_blank"
      >Print preview</a
    >`
  }
  /* eslint-enable lit/attribute-value-entities */

  private renderTabGroup() {
    const closeTab = (event: SlCloseEvent) => {
      const tab = event.target as SlTab
      this.shadowRoot.dispatchEvent(
        new CustomEvent('stencila-close-document', {
          bubbles: true,
          composed: true,
          detail: { docId: tab.panel },
        })
      )
    }

    const baseTabStyles = apply([
      'relative',
      'mr-1',
      'bg-grey-200',
      'border border-gray-200 border-b-0 rounded-t',
    ])

    const tabClasses = css`
      &::part(base) {
        color: #171817;
        height: 2rem;
        font-size: 14px;
        font-weight: 500;
      }
      &[active] {
        background-color: #ffffff;

        &::part(base) {
          font-weight: 400;
        }

        &::after {
          content: '';
          position: absolute;
          height: 1px;
          width: 100%;
          background-color: #ffffff;
          left: 0px;
          bottom: -1px;
        }
      }
    `

    // disable the 'active-tab-indicator'
    const tabGroupClasses = css`
      &::part(active-tab-indicator) {
        display: none;
      }
      &::part(tabs) {
        border-bottom: 1px solid #dedede;
      }
    `

    const tabPanelClasses = css`
      &::part(base) {
        padding-top: 0px;
      }
    `

    const content = this.docs
      ? this.docs.map(
          ({ docId, name }) => html`
            <sl-tab
              slot="nav"
              class="${baseTabStyles} ${tabClasses}"
              panel=${docId}
              ?active=${docId === this.activeDoc}
              closable
            >
              <!-- TODO: disambiguate files with same name in different folders -->
              ${name}
            </sl-tab>
            <sl-tab-panel
              name=${docId}
              ?active=${docId === this.activeDoc}
              class="${tabPanelClasses}"
            >
              <stencila-ui-view-container
                view=${this.contextObject.currentView}
              >
                ${this.renderDocumentView(docId)}
              </stencila-ui-view-container>
            </sl-tab-panel>
          `
        )
      : // TODO: Render a welcome screen
        // See https://github.com/stencila/stencila/issues/2027
        html`Welcome!`

    return html`<sl-tab-group
      @sl-close=${(event: SlCloseEvent) => closeTab(event)}
      class="${tabGroupClasses}"
      >${content}</sl-tab-group
    >`
  }

  private renderDocumentView(docId: DocumentId) {
    switch (this.contextObject.currentView) {
      case 'static':
        return html`<stencila-static-view
          view="static"
          doc=${docId}
          theme=${this.theme}
          fetch
        ></stencila-static-view>`

      case 'live':
        return html`<stencila-live-view
          view="live"
          doc=${docId}
          theme=${this.theme}
        ></stencila-live-view>`

      case 'dynamic':
        return html`<stencila-dynamic-view
          view="dynamic"
          doc=${docId}
          theme=${this.theme}
        ></stencila-dynamic-view>`

      case 'source':
        return html`<stencila-source-view
          view="source"
          doc=${docId}
          format=${this.format}
        ></stencila-source-view>`

      case 'split':
        return html`<stencila-split-view
          view="split"
          doc=${docId}
          format=${this.format}
          theme=${this.theme}
        ></stencila-split-view>`

      case 'visual':
        return html`<stencila-visual-view
          view="visual"
          doc=${docId}
          theme=${this.theme}
        ></stencila-visual-view>`

      case 'directory':
        return html`<stencila-live-view
          view="live"
          doc=${docId}
          theme=${this.theme}
        ></stencila-live-view>`

      default:
        return html``
    }
  }

  override connectedCallback() {
    super.connectedCallback()

    // Instantiate context passing through attribute values where possible.
    // This allows query parameters e.g ?view=source to be effective.
    this.contextObject = {
      currentView: this.view,
      directoryOpen: true,
    }

    // Instantiate the info context based on the view
    this.infoViewContext = {
      infoViewOpen: this.view === 'source' ? true : false,
    }

    // Event listener for updating whether the directory view is open or closed
    this.shadowRoot.addEventListener(
      'stencila-directory-toggle',
      (
        e: Event & { detail: Required<Pick<SidebarContext, 'directoryOpen'>> }
      ) => {
        this.contextObject = {
          ...this.contextObject,
          directoryOpen: e.detail.directoryOpen,
        }
      }
    )

    // Event listener for opening a document.
    // Makes the opened document the active document.
    // If the document is already open, makes it the active document.
    this.shadowRoot.addEventListener(
      'stencila-open-document',
      async (e: Event & { detail: File }) => {
        const file = e.detail

        const index = this.docs.findIndex((doc) => doc.path == file.path)
        if (index < 0) {
          const docId = await DirectoryView.openPath(file.path)
          this.docs.push({ docId, ...file })
          this.activeDoc = docId
          this.requestUpdate()
        } else {
          const docId = this.docs[index].docId
          if (docId !== this.activeDoc) {
            this.activeDoc = docId
            this.requestUpdate()
          }
        }
      }
    )

    // Event listener for closing a document
    // If the closed document is the active document, makes the document
    // before it the active document (unless closing the very fast document)
    this.shadowRoot.addEventListener(
      'stencila-close-document',
      async (e: Event & { detail: { docId: DocumentId } }) => {
        const { docId } = e.detail

        const index = this.docs.findIndex((doc) => doc.docId === docId)
        if (index > -1) {
          this.docs.splice(index, 1)
          if (docId === this.activeDoc) {
            if (index == 0) {
              this.activeDoc =
                this.docs.length === 0 ? null : this.docs[0].docId
            } else {
              this.activeDoc = this.docs[index - 1].docId
            }
          }
          this.requestUpdate()
        }

        // Close the document on the server. This is a clean up task
        // which should not block the rendering so is not awaited here.
        DirectoryView.closeDocument(docId)
      }
    )

    // Event listener for changing the view
    this.shadowRoot.addEventListener(
      'stencila-view-change',
      (
        e: Event & { detail: Required<Pick<SidebarContext, 'currentView'>> }
      ) => {
        this.contextObject = {
          ...this.contextObject,
          currentView: e.detail.currentView,
        }
      }
    )

    this.shadowRoot.addEventListener(
      'stencila-infoview-node',
      (
        e: Event & { detail: Required<Pick<InfoViewContext, 'currentNodeId'>> }
      ) => {
        this.infoViewContext = {
          ...this.infoViewContext,
          currentNodeId: e.detail.currentNodeId,
        }
      }
    )
  }
}
