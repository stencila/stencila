import { EntityId } from '@reduxjs/toolkit'
import { Component, Element, h, Host, Prop, Watch } from '@stencil/core'
import { option as O } from 'fp-ts'
import { pipe } from 'fp-ts/function'
import { DocumentEvent } from 'stencila'
import { CHANNEL } from '../../../../preload/channels'
import { state } from '../../../../renderer/store'
import { selectDoc } from '../../../../renderer/store/documentPane/documentPaneSelectors'
import { saveEditorState } from '../../../../renderer/store/editorState/editorStateActions'
import { editorStateById } from '../../../../renderer/store/editorState/editorStateSelectors'
import { EditorState } from '../../../../renderer/store/editorState/editorStateTypes'
import { client } from '../../../client'
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
      const editor = await this.editorRef.getRef()
      return saveEditorState(documentId)({
        id: documentId,
        state: editor.state,
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
        client.documents.contents(documentId).then(({ value }) => {
          this.editorRef?.setState(value, {
            language: this.fileFormatToLanguage(),
          })
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
    this.editorRef?.getRef().then((editor) => {
      editor.setState(state.state)
    })
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

    this.restoreOrCreateDocState(documentId)
      .then(() => this.editorRef?.getRef())
      .then((editor) => {
        // Return input focus to editor so that user can type into the editor right away
        editor?.focus()
      })
  }

  private unsubscribeFromDocument = (documentId: EntityId) => {
    window.api.removeAll(CHANNEL.DOCUMENT_WRITE_ACTIVE)
    window.api.removeAll(CHANNEL.DOCUMENTS_DUMP)

    return client.documents.unsubscribe({
      documentId,
      topics: ['modified'],
    })
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
        client.documents.write({
          documentId: this.documentId,
          content: text,
        })
      })
      .catch((err) => {
        errorToast(err)
      })
  }

  componentWillLoad() {
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
            isControlled={true}
          ></stencila-editor>
        </div>
      </Host>
    )
  }
}
