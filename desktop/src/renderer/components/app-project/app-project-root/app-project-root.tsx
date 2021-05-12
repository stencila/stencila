import { Component, h } from '@stencil/core'
import { match, Route } from '@stencil/router'
import { ProjectRouter } from '../projectRouter'

@Component({
  tag: 'app-project-root',
  styleUrl: 'app-project-root.css',
  scoped: true,
})
export class AppProjectRoot {
  render() {
    return (
      <div class="projectWindow">
        <ProjectRouter.Switch>
          <Route
            path={match('/project/:projectDir*')}
            render={({ projectDir = '' }) => [
              <app-project-sidebar-files
                projectDir={decodeURI(projectDir)}
              ></app-project-sidebar-files>,
              <main>
                <app-project-file-preview></app-project-file-preview>
              </main>,
            ]}
          ></Route>
        </ProjectRouter.Switch>
      </div>
    )
  }
}
