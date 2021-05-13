import { Component, h } from '@stencil/core'
import { CHANNEL } from '../../../preload/index'

@Component({
  tag: 'app-launcher',
  styleUrl: 'app-launcher.css',
  scoped: true,
})
export class AppLauncher {
  private selectFiles = () => {
    window.api.invoke(CHANNEL.SELECT_PROJECT_DIR)
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
