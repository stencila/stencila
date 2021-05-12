import { Component, h } from '@stencil/core'
import { CHANNEL } from '../../../preload/index'

@Component({
  tag: 'app-launcher',
  styleUrl: 'app-launcher.css',
  scoped: true,
})
export class AppLauncher {
  private selectFiles = () => {
    window.api.invoke(CHANNEL.SELECT_PROJECT_DIR).then((selectedFiles) => {
      // TODO: Get type inference on IPC calls
      if (
        typeof selectedFiles === 'object' &&
        Object.prototype.hasOwnProperty.call(selectedFiles, 'filePaths')
      ) {
        // @ts-ignore
        const path = selectedFiles.filePaths
          ? // @ts-ignore
            selectedFiles?.filePaths[0]
          : undefined

        if (path) {
          window.api.invoke(CHANNEL.SHOW_PROJECT_WINDOW, path)
        }
      }
    })
  }

  render() {
    return (
      <div class="app-home">
        <h1>Stencila</h1>

        <main>
          <div>
            <stencila-button>New document</stencila-button>
            <stencila-button>New project</stencila-button>

            <hr />

            <stencila-button onClick={this.selectFiles}>
              Open folder…
            </stencila-button>
          </div>

          <hr />

          <h2>Recent projects</h2>

          <hr />
        </main>
      </div>
    )
  }
}
