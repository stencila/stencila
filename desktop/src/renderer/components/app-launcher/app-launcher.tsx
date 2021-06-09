import { Component, h, State } from '@stencil/core'
import Logo from '@stencila/brand/dist/logos/stencilaLogo.svg'
import { CHANNEL } from '../../../preload/index'
import { fetchRecentProjects } from '../../store/project/projectActions'

@Component({
  tag: 'app-launcher',
  styleUrl: 'app-launcher.css',
  scoped: true,
})
export class AppLauncher {
  @State() recentProjects: string[] = []

  private selectFiles = () => {
    window.api.invoke(CHANNEL.SELECT_PROJECT_DIR)
  }

  private openProject = (path: string) => (e: Event) => {
    e.preventDefault()
    window.api.invoke(CHANNEL.OPEN_PROJECT, path).then(() => {
      window.api.invoke(CHANNEL.CLOSE_LAUNCHER_WINDOW)
    })
  }

  componentWillLoad() {
    this.recentProjects = fetchRecentProjects()
  }

  render() {
    return (
      <div class="app-home">
        <img src={Logo} class="logo" />

        <main>
          <div class="launcherActions">
            <stencila-button
              size="small"
              fill={true}
              onClick={this.selectFiles}
            >
              Open folder…
            </stencila-button>

            <stencila-button size="small" fill={true} disabled={true}>
              New document
            </stencila-button>

            <stencila-button size="small" fill={true} disabled={true}>
              New project
            </stencila-button>
          </div>

          <div class="recentProjects">
            <h2>Recent projects</h2>
            <ul>
              {this.recentProjects.map((projectPath) => (
                <li>
                  <a
                    onClick={this.openProject(projectPath)}
                    class="recentProjectItem"
                  >
                    <stencila-icon icon="folder"></stencila-icon>
                    {projectPath}
                  </a>
                </li>
              ))}
            </ul>
          </div>
        </main>
      </div>
    )
  }
}
