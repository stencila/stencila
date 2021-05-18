import { Component, Element, h, Host, Prop, Watch } from '@stencil/core'
import { File } from 'stencila'
import { CHANNEL } from '../../../../preload'
import { state } from '../../../store'
import { selectProjectFile } from '../../../store/project/projectSelectors'

@Component({
  tag: 'app-document-preview',
  styleUrl: 'app-document-preview.css',
  // Scoped must be off for this component to avoid mangling class names
  // for the CodeEditor selectors.
  scoped: false,
})
export class AppDocumentPreview {
  @Element() el: HTMLElement

  private editorRef: HTMLStencilaEditorElement | null = null

  @Prop() filePath: string

  private file?: File

  @Watch('filePath')
  filePathWatchHandler(newValue: string, prevValue: string) {
    if (newValue !== prevValue) {
      this.updateEditorContents()
    }
  }

  private updateEditorContents = () => {
    this.file = selectProjectFile(state)(this.filePath)

    window.api
      .invoke(CHANNEL.GET_DOCUMENT_CONTENTS, this.filePath)
      .then((contents) => {
        if (typeof contents === 'string') {
          this.editorRef?.setContents(contents)
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
    this.updateEditorContents()
  }

  render() {
    return (
      <Host>
        <div class="app-document-preview">
          <stencila-editor
            activeLanguage={this.fileFormatToLanguage()}
          ></stencila-editor>
        </div>
      </Host>
    )
  }
}
