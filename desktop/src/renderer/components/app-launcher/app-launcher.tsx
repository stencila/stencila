import { Component, h, State } from '@stencil/core'
import Logo from '@stencila/brand/dist/logos/stencilaLogo.svg'
import { client } from '../../client'
import { fetchRecentProjects } from '../../store/project/projectActions'
import { userOSPathSeparator } from '../../utils/env'

const projectDirName = (path: string): string | undefined =>
  path.split(userOSPathSeparator).pop()

@Component({
  tag: 'app-launcher',
  styleUrl: 'app-launcher.css',
  scoped: true,
})
export class AppLauncher {
  private appVersion: string

  @State() recentProjects: string[] = []

  private openProject = (path: string) => async (e: Event) => {
    e.preventDefault()
    await client.projects.open(path)
    client.launcher.close()
  }

  componentWillLoad() {
    this.recentProjects = fetchRecentProjects()
    return client.app.version().then(({ value }) => {
      this.appVersion = value
    })
  }

  render() {
    return (
      <div class="app-launcher">
        <main>
          <div class="launcherActions">
            <div class="primaryActions">
              <div class="logo">
                <img src={Logo} />
              </div>

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

            <div class="secondaryActions">
              <stencila-button
                icon="settings-3"
                iconOnly={true}
                minimal={true}
                size="small"
                color="neutral"
                tooltip="Settings"
                onClick={(e) => {
                  e.preventDefault()
                  client.config.window.open()
                }}
              >
                Settings
              </stencila-button>

              <p class="appVersion">v{this.appVersion}</p>
            </div>
          </div>

          <div class="recentProjects">
            <h2>Recent projects</h2>
            <ul>
              {this.recentProjects.map((projectPath) => {
                const projectName = projectDirName(projectPath)
                return (
                  <li>
                    <a
                      onClick={this.openProject(projectPath)}
                      class="recentProjectItem"
                    >
                      <stencila-icon icon="folder"></stencila-icon>
                      <div class="meta">
                        <h3 class="name" title={projectName ?? projectPath}>
                          {projectName ?? projectPath}
                        </h3>
                        {projectName && <h4 class="path">{projectPath}</h4>}
                      </div>
                    </a>
                  </li>
                )
              })}
            </ul>
          </div>
        </main>
      </div>
    )
  }
}
