import { EntityId } from '@reduxjs/toolkit'
import { Component, Fragment, h, Host, Prop, State, Watch } from '@stencil/core'
import { option as O } from 'fp-ts'
import { pipe } from 'fp-ts/function'
import { File } from 'stencila'
import { state } from '../../../store'
import { selectDoc } from '../../../store/documentPane/documentPaneSelectors'
import { updateProjectSettings } from '../../../store/project/projectSelectors'
import { SessionsStoreKeys, sessionStore } from '../../../store/sessionStore'

const themes = {
  elife: 'eLife',
  giga: 'GigaScience',
  latex: 'LaTeX',
  stencila: 'Stencila',
  tufte: 'Tufte',
  wilmore: 'Wilmore',
}

@Component({
  tag: 'app-document-preview',
  styleUrl: 'app-document-preview.css',
  shadow: true,
})
export class AppDocumentPreview {
  /**
   * ID of the document to be previewed
   */
  @Prop() documentId: EntityId

  private updateDoc = (id: EntityId) => {
    const doc = selectDoc(state)(id)
    if (doc) {
      this.doc = doc
    }
  }

  @Watch('documentId')
  documentIdWatchHandler(newValue: string, prevValue: string) {
    if (newValue !== prevValue) {
      this.updateDoc(newValue)
    }
  }

  @State() doc: File

  @State() serverUrl: URL | undefined

  @State() theme = 'stencila'

  private onThemeChange = (e: Event) => {
    this.theme = (e.target as HTMLSelectElement).value
    updateProjectSettings({ theme: this.theme })
  }

  private getServerUrl = (): URL | undefined =>
    pipe(
      sessionStore.get(SessionsStoreKeys.SERVER_URL),
      O.map((serverUrl) => {
        window.clearInterval(this.serverUrlPollRef)
        this.serverUrl = new window.URL(serverUrl)
        return this.serverUrl
      }),
      O.getOrElseW(() => undefined)
    )

  private serverUrlPollRef: number
  private pollForServerUrl = () => {
    this.serverUrlPollRef = window.setInterval(() => this.getServerUrl(), 200)
  }

  componentWillLoad() {
    this.updateDoc(this.documentId)
    this.getServerUrl()
    if (this.serverUrl === undefined) {
      this.pollForServerUrl()
    }
  }

  render() {
    return (
      <Host>
        <div class="app-document-preview">
          {this.serverUrl === undefined ? (
            'Loading'
          ) : (
            <Fragment>
              <iframe
                title="document-preview"
                src={`${this.serverUrl.origin}${this.doc.path}${this.serverUrl.search}&theme=${this.theme}`}
              />

              <menu>
                <label aria-label="Project Theme">
                  <stencila-icon icon="palette"></stencila-icon>
                  <select onChange={this.onThemeChange}>
                    {Object.entries(themes).map(([value, name]) => (
                      <option value={value} selected={value === this.theme}>
                        {name}
                      </option>
                    ))}
                  </select>
                </label>
              </menu>
            </Fragment>
          )}
        </div>
      </Host>
    )
  }
}
