import { EntityId } from '@reduxjs/toolkit'
import { Component, Element, h, Host, Prop, State, Watch } from '@stencil/core'
import { DocumentEvent } from 'stencila'
import { CHANNEL } from '../../../../preload/channels'
import { state } from '../../../../renderer/store'
import { selectDoc } from '../../../../renderer/store/documentPane/documentPaneSelectors'

@Component({
  tag: 'app-document-editor',
  styleUrl: 'app-document-editor.css',
  // Scoped must be off for this component to avoid mangling class names
  // for the CodeEditor selectors.
  scoped: false,
})
export class AppDocumentEditor {
  @Element() el: HTMLElement

  private editorRef: HTMLStencilaEditorElement | null = null

  @Prop() documentId: EntityId

  @Watch('documentId')
  documentIdWatchHandler(newValue: string, prevValue: string) {
    if (newValue !== prevValue) {
      this.unsubscribeFromDocument(prevValue).then(() => {
        this.subscribeToDocument(newValue)
      })
    }
  }

  @State() contents: string

  private subscribeToDocument = (documentId = this.documentId) => {
    window.api.receive(CHANNEL.GET_DOCUMENT_CONTENTS, (event) => {
      const { type, content } = event as DocumentEvent
      if (type === 'modified' && typeof content == 'string') {
        this.contents = content
      }
    })

    window.api.receive(CHANNEL.SAVE_ACTIVE_DOCUMENT, () => {
      this.saveDoc()
    })

    return window.api
      .invoke(CHANNEL.GET_DOCUMENT_CONTENTS, documentId)
      .then((contents) => {
        if (typeof contents === 'string') {
          this.contents = contents
        }
      })
  }

  private unsubscribeFromDocument = (documentId = this.documentId) => {
    window.api.removeAll(CHANNEL.SAVE_ACTIVE_DOCUMENT)
    window.api.removeAll(CHANNEL.GET_DOCUMENT_CONTENTS)

    return window.api.invoke(CHANNEL.UNSUBSCRIBE_DOCUMENT, {
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
        window.api.invoke(CHANNEL.SAVE_DOCUMENT, {
          documentId: this.documentId,
          content: text,
        })
      })
      .catch((err) => {
        console.log(err)
      })
  }

  componentWillLoad() {
    return this.subscribeToDocument()
  }

  componentDidLoad() {
    this.editorRef = this.el.querySelector('stencila-editor')
  }

  disconnectedCallback() {
    this.unsubscribeFromDocument()
  }

  render() {
    return (
      <Host>
        <div class="app-document-editor">
          <stencila-editor
            activeLanguage={this.fileFormatToLanguage()}
            contents={this.contents}
          ></stencila-editor>
        </div>
      </Host>
    )
  }
}
