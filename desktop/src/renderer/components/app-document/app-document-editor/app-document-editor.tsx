import { EntityId } from '@reduxjs/toolkit'
import { Component, Element, h, Host, Prop, Watch } from '@stencil/core'
import { FileFormatUtils } from '@stencila/components'
import { debounce } from 'debounce'
import { option as O } from 'fp-ts'
import { pipe } from 'fp-ts/function'
import { DocumentEvent } from 'stencila'
import { CHANNEL } from '../../../../preload/channels'
import { state } from '../../../../renderer/store'
import { selectDoc } from '../../../../renderer/store/documentPane/documentPaneSelectors'
import { saveEditorState } from '../../../../renderer/store/editorState/editorStateActions'
import { editorStateById } from '../../../../renderer/store/editorState/editorStateSelectors'
import { EditorState } from '../../../../renderer/store/editorState/editorStateTypes'
import { alterDocument, saveDocument } from '../../../actions/documentActions'
import { client } from '../../../client'
import { configState } from '../../../store/appConfig'
import {
  openDocumentInActivePane,
  patchDocument,
} from '../../../store/documentPane/documentPaneActions'
import { errorToast } from '../../../utils/errors'

@Component({
  tag: 'app-document-editor',
  styleUrl: 'app-document-editor.css',
  // Scoped must be off for this component to avoid mangling class names
  // for the CodeEditor selectors.
  scoped: false,
})
export class AppDocumentEditor {
  @Element() el: HTMLElement

  private editorRef: HTMLStencilaEditorElement | undefined

  @Prop() documentId: EntityId

  @Watch('documentId')
  async documentIdWatchHandler(newDocId: string, prevDocId: string) {
    if (newDocId !== prevDocId) {
      return this.saveDocState(prevDocId)
        .then(() => this.unsubscribeFromDocument(prevDocId))
        .then(() => this.subscribeToDocument(newDocId))
    }
  }

  /**
   * Persist internal editor state into global store.
   * This allows us to keep history and other state on a per-document basis.
   */
  private saveDocState = async (
    documentId: EntityId
  ): Promise<EditorState | void> => {
    if (this.editorRef) {
      const editorState = await this.editorRef.getState()
      return saveEditorState(documentId)({
        id: documentId,
        state: editorState,
      })
    }
  }

  /**
   * Attempt to retrieve editor state from global store.
   * If document is being opened for the first time, then read contents from disk instead.
   */
  private restoreOrCreateDocState = async (
    documentId: EntityId
  ): Promise<void> => {
    pipe(
      documentId,
      editorStateById,
      O.map(this.setDocState),
      O.getOrElse(() => {
        client.documents.contents(documentId).then(({ value: contents }) => {
          this.editorRef?.setStateFromString(contents)
        })
      })
    )
  }

  /**
   * Completely replace editor state with given state, including the stored configuration,
   * edit history, language, etc.
   * Note that this is different to the `setDocContents` function.
   */
  private setDocState = (state: EditorState) => {
    this.editorRef?.setState(state.state)
  }

  /**
   * Replace editor contents while preserving existing state (edit history, extensions, language, etc.)
   */
  private setDocContents = (contents: string) => {
    this.editorRef?.setContents(contents)
  }

  private subscribeToDocument = (documentId: EntityId) => {
    // Listen to file events and update contents
    window.api.receive(CHANNEL.DOCUMENTS_DUMP, (event) => {
      const { type, content } = event as DocumentEvent
      if (type === 'modified' && typeof content == 'string') {
        // TODO: Ask user if they want to update document contents
        this.setDocContents(content)
      }
    })

    // Handle global file save events, both keyboard shortcut and File menu items
    window.api.receive(CHANNEL.DOCUMENT_WRITE_ACTIVE, this.saveDoc)
    window.api.receive(CHANNEL.DOCUMENT_WRITE_ACTIVE_AS, this.saveDocAs)

    this.restoreOrCreateDocState(documentId)
      .then(() => this.editorRef?.getRef())
      .then((editor) => {
        // Return input focus to editor so that user can type into the editor right away
        editor?.focus()
      })
  }

  private unsubscribeFromDocument = (documentId: EntityId) => {
    window.api.removeAll(CHANNEL.DOCUMENT_WRITE_ACTIVE)
    window.api.removeAll(CHANNEL.DOCUMENT_WRITE_ACTIVE_AS)
    window.api.removeAll(CHANNEL.DOCUMENTS_DUMP)

    return client.documents.unsubscribe({
      documentId,
      topics: ['modified'],
    })
  }

  /**
   * The number of milliseconds to wait between consecutive calls of the document update handler.
   * Allows us to avoid generating too many previews as the user is typing, and degrading performance.
   */
  onDocChangeTimeout = 300

  /**
   * Function to call whenever the contents of the editor change.
   */
  private onDocChange = debounce(() => {
    this.editorRef
      ?.getContents()
      .then((contents) => client.documents.load(this.documentId, contents.text))
      .then(() => {
        if (selectDoc(state)(this.documentId)?.status === 'synced') {
          patchDocument({
            id: this.documentId,
            status: 'unwritten',
          })
        }
      })
  }, this.onDocChangeTimeout)

  private onSetLanguage = (e: CustomEvent<FileFormatUtils.FileFormat>) => {
    if (e.detail.ext) {
      alterDocument(this.documentId, undefined, e.detail.ext)
    }
  }

  private fileFormatToLanguage = (): string => {
    const file = selectDoc(state)(this.documentId)
    switch (file?.format?.name) {
      case 'ipynb':
        return 'python'
      default:
        return file?.format?.name ?? 'md'
    }
  }

  private saveDoc = () => {
    this.editorRef
      ?.getContents()
      .then(({ text }) => {
        saveDocument(this.documentId, text)
      })
      .catch((err) => {
        errorToast(err)
      })
  }

  private saveDocAs = async () => {
    const { value: maybeFilePath } = await client.documents.createFilePath()
    const contents = await this.editorRef?.getContents()

    if (maybeFilePath.canceled || !contents) return

    client.documents
      .create(maybeFilePath.filePath)
      .then(({ value: doc }) => {
        saveDocument(
          doc.id,
          contents.text,
          FileFormatUtils.lookupFormat(contents.language).ext ??
            contents.language
        )
      })
      .then(openDocumentInActivePane(maybeFilePath.filePath))
  }

  componentDidLoad() {
    return this.subscribeToDocument(this.documentId)
  }

  disconnectedCallback() {
    return this.unsubscribeFromDocument(this.documentId)
  }

  render() {
    return (
      <Host>
        <div class="app-document-editor">
          <stencila-editor
            ref={(el) => (this.editorRef = el)}
            activeLanguage={this.fileFormatToLanguage()}
            lineNumbers={configState.global.editors?.lineNumbers}
            lineWrapping={configState.global.editors?.lineWrapping}
            contentChangeHandler={this.onDocChange}
            onSetLanguage={this.onSetLanguage}
          ></stencila-editor>
        </div>
      </Host>
    )
  }
}
