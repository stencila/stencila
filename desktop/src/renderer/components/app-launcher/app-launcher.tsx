import { Component, h, State } from '@stencil/core'
import Logo from '@stencila/brand/dist/logos/stencilaLogo.svg'
import { client } from '../../client'
import { fetchRecentProjects } from '../../store/project/projectActions'

@Component({
  tag: 'app-launcher',
  styleUrl: 'app-launcher.css',
  scoped: true,
})
export class AppLauncher {
  @State() recentProjects: string[] = []

  private openProject = (path: string) => async (e: Event) => {
    e.preventDefault()
    await client.projects.open(path)
    client.launcher.close()
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
              onClick={client.projects.openUsingPicker}
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
