import { Component, h, State } from '@stencil/core'
import Logo from '@stencila/brand/dist/logos/stencilaLogo.svg'
import { captureError } from '../../../preload/errors'
import { client } from '../../client'
import { fetchRecentProjects } from '../../store/project/projectActions'
import { userOSPathSeparator } from '../../utils/env'
import { showAndCaptureError } from '../../utils/errors'

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

  private pickAndOpenProject = async (e: Event) => {
    e.preventDefault()
    client.projects
      .openUsingPicker()
      .then(({ value }) => {
        if (!value.canceled) {
          client.launcher.close().catch((err) => captureError(err))
        }
      })
      .catch((err) => showAndCaptureError(err))
  }

  private openProject = (path: string) => async (e: Event) => {
    e.preventDefault()
    await client.projects.open(path)
    client.launcher.close().catch((err) => showAndCaptureError(err))
  }

  private openSettings(): ((event: MouseEvent) => void) | undefined {
    return (e) => {
      e.preventDefault()
      client.config.window.open().catch((err) => showAndCaptureError(err))
    }
  }

  componentWillLoad() {
    this.recentProjects = fetchRecentProjects()
    return client.app.version().then(({ value: version }) => {
      this.appVersion = version
    })
  }

  render() {
    return (
      <div class="app-launcher">
        <main>
          <div class="launcherActions">
            <div class="primaryActions">
              <div class="logo">
                <img alt="Stencila logo" src={Logo} />
              </div>

              <stencila-button
                size="small"
                fill={true}
                onClick={this.pickAndOpenProject}
              >
                Open folder…
              </stencila-button>

              <stencila-button
                size="small"
                fill={true}
                onClick={client.projects.new}
              >
                New project…
              </stencila-button>

              <stencila-button size="small" fill={true} disabled={true}>
                New document
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
                onClick={this.openSettings()}
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
                        {projectName !== undefined && (
                          <h4 class="path">{projectPath}</h4>
                        )}
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
