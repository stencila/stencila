import { Component, h, Host, Prop, State } from '@stencil/core'
import { CHANNEL } from '../../../../../preload/index'

@Component({
  tag: 'app-project-sidebar-files',
  styleUrl: 'app-project-sidebar-files.css',
  scoped: true,
})
export class AppProjectSidebarFiles {
  @State()
  private files: string[] = []

  @Prop()
  projectDir: string

  private getFileList = (path: string) => {
    window.api.invoke(CHANNEL.GET_PROJECT_FILES, path).then((files) => {
      // TODO: Get type inference on IPC calls
      if (Array.isArray(files) && files.every((i) => typeof i === 'string')) {
        this.files = files
      }
    })
  }

  componentWillLoad() {
    return this.getFileList(`/${this.projectDir}`)
  }

  render() {
    return (
      <Host>
        <div class="app-project-sidebar-files">
          <ul>
            {this.files.map((file) => (
              <li title={file}>{file}</li>
            ))}
          </ul>
        </div>
      </Host>
    )
  }
}
