import { EntityId } from '@reduxjs/toolkit'
import { Component, Element, h, Host, Prop, Watch } from '@stencil/core'
import { DocumentEvent, File } from 'stencila'
import { CHANNEL } from '../../../../preload'

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

  private file?: File

  private closeDoc = (documentId = this.documentId) =>
    window.api.invoke(CHANNEL.UNSUBSCRIBE_DOCUMENT, {
      documentId,
      topics: ['modified'],
    })

  @Watch('documentId')
  documentIdWatchHandler(newValue: string, prevValue: string) {
    if (newValue !== prevValue) {
      this.closeDoc(prevValue).then(() => {
        this.subscribeToUpdates(newValue)
      })
    }
  }

  private subscribeToUpdates = (documentId = this.documentId) => {
    window.api
      .invoke(CHANNEL.GET_DOCUMENT_CONTENTS, documentId)
      .then((contents) => {
        if (typeof contents === 'string') {
          this.editorRef?.setContents(contents)
        }
      })

    window.api.receive(CHANNEL.GET_DOCUMENT_CONTENTS, (event) => {
      const { type, content } = event as DocumentEvent
      if (type === 'modified' && typeof content == 'string') {
        this.editorRef?.setContents(content)
      }
    })

    window.api.receive(CHANNEL.SAVE_ACTIVE_DOCUMENT, () => {
      this.saveDoc()
    })
  }

  private fileFormatToLanguage = (): string => {
    switch (this.file?.format) {
      case 'bash':
        return 'bash'
      case 'py':
      case 'ipynb':
        return 'python'
      default:
        return 'r'
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

  componentDidLoad() {
    this.editorRef = this.el.querySelector('stencila-editor')
    this.subscribeToUpdates()
  }

  render() {
    return (
      <Host>
        <div class="app-document-editor">
          <stencila-editor
            activeLanguage={this.fileFormatToLanguage()}
          ></stencila-editor>
        </div>
      </Host>
    )
  }
}
