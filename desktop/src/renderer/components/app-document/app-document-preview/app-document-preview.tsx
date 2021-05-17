import { Component, Element, h, Host, Prop, Watch } from '@stencil/core'
import { CHANNEL } from '../../../../preload'

@Component({
  tag: 'app-document-preview',
  styleUrl: 'app-document-preview.css',
  scoped: true,
})
export class AppDocumentPreview {
  @Element() el: HTMLElement

  private editorRef: HTMLStencilaEditorElement | null = null

  @Prop() filePath: string

  private updateEditorContents = () => {
    window.api
      .invoke(CHANNEL.GET_DOCUMENT_CONTENTS, this.filePath)
      .then((contents) => {
        if (typeof contents === 'string') {
          this.editorRef?.setContents(contents)
        }
      })
  }

  @Watch('filePath')
  filePathWatchHandler(newValue: string, prevValue: string) {
    if (newValue !== prevValue) {
      this.updateEditorContents()
    }
  }

  componentDidLoad() {
    this.editorRef = this.el.querySelector('stencila-editor')
  }

  render() {
    return (
      <Host>
        <div class="app-document-preview">
          <stencila-editor>
            <div slot="text">{this.filePath} contents go here</div>
          </stencila-editor>
        </div>
      </Host>
    )
  }
}
