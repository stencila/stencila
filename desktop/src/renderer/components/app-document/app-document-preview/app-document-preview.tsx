import { EntityId } from '@reduxjs/toolkit'
import { Component, h, Host, Prop, State, Watch } from '@stencil/core'
import { option as O } from 'fp-ts'
import { pipe } from 'fp-ts/function'
import { File } from 'stencila'
import { state } from '../../../store'
import { selectDoc } from '../../../store/documentPane/documentPaneSelectors'
import { SessionsStoreKeys, sessionStore } from '../../../store/sessionStore'

const themes = {
  elife: 'eLife',
  f1000: 'f1000',
  giga: 'GigaScience',
  latex: 'latex',
  nature: 'nature',
  plos: 'PLoS',
  stencila: 'Stencila',
  tufte: 'tufte',
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

  @State() serverUrl: URL

  @State() theme = 'stencila'

  private onThemeChange = (e: Event) => {
    this.theme = (e.target as HTMLSelectElement).value
  }

  componentWillLoad() {
    this.updateDoc(this.documentId)

    pipe(
      sessionStore.get(SessionsStoreKeys.SERVER_URL),
      O.map((serverUrl) => {
        this.serverUrl = new window.URL(serverUrl)
      })
    )
  }

  render() {
    return (
      <Host>
        <div class="app-document-preview">
          <iframe
            title="document-preview"
            src={`${this.serverUrl.origin}/${this.doc.path}${this.serverUrl.search}&theme=${this.theme}`}
            sandbox="allow-scripts"
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
        </div>
      </Host>
    )
  }
}
