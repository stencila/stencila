import { Component, h } from '@stencil/core'
import { CHANNEL } from '../../../preload/index'
import { Router } from '../../router'

@Component({
  tag: 'app-home',
  styleUrl: 'app-home.css',
  shadow: true,
})
export class AppHome {
  private selectFiles = () => {
    window.api.invoke(CHANNEL.SELECT_DIRS).then((selectedFiles) => {
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
          const p = `/project${path}`
          console.log(p)
          Router.push(p)
        }
      }
    })
  }

  render() {
    return (
      <div class="app-home">
        <h1>Stencila</h1>

        <stencila-button>New document</stencila-button>
        <stencila-button>New project</stencila-button>

        <hr />

        <stencila-button onClick={this.selectFiles}>Open…</stencila-button>

        <hr />

        <h2>Recent projects</h2>

        <hr />
      </div>
    )
  }
}
