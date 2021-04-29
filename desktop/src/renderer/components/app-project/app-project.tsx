import { Component, h, Prop, State } from '@stencil/core'
import { href } from '@stencil/router'
import { CHANNEL } from '../../../preload/index'

@Component({
  tag: 'app-project',
  styleUrl: 'app-project.css',
  shadow: true,
})
export class AppProject {
  @State()
  private files: string[] = []

  @Prop()
  projectDir: string

  private getFileList = (path: string) => {
    window.api.invoke(CHANNEL.READ_DIR, path).then((files) => {
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
      <div class="app-project">
        <div>
          <h2>Project Files</h2>
          <a {...href('/')}>Go back</a>
        </div>

        <strong>/{this.projectDir}</strong>

        <ul>
          {this.files.map((file) => (
            <li>{file}</li>
          ))}
        </ul>
      </div>
    )
  }
}
