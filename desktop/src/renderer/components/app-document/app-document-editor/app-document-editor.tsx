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

  @Prop() filePath: string

  private file?: File

  private closeDoc = (filePath = this.filePath) =>
    window.api.invoke(CHANNEL.CLOSE_DOCUMENT, filePath)

  @Watch('filePath')
  filePathWatchHandler(newValue: string, prevValue: string) {
    if (newValue !== prevValue) {
      this.closeDoc(prevValue).then(() => {
        this.subscribeToUpdates(newValue)
      })
    }
  }

  private subscribeToUpdates = (filePath = this.filePath) => {
    window.api
      .invoke(CHANNEL.GET_DOCUMENT_CONTENTS, filePath)
      .then((contents) => {
        if (typeof contents === 'string') {
          this.editorRef?.setContents(contents)
        }
      })

    window.api.receive(CHANNEL.GET_DOCUMENT_CONTENTS, (event) => {
      const e = event as DocumentEvent
      if (e.type === 'modified') {
        this.editorRef?.setContents(e.content)
      }
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
