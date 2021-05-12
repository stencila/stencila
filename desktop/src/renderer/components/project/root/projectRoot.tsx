import { Component, h } from '@stencil/core'
import { match, Route } from '@stencil/router'
import { ProjectRouter } from '../projectRouter'

@Component({
  tag: 'project-root',
  styleUrl: 'projectRoot.css',
  scoped: true,
})
export class AppProject {
  render() {
    return (
      <div class="projectWindow">
        <ProjectRouter.Switch>
          <Route
            path={match('/project/:projectDir*')}
            render={({ projectDir = '' }) => [
              <project-sidebar-files
                projectDir={decodeURI(projectDir)}
              ></project-sidebar-files>,
              <main>
                <project-file-preview></project-file-preview>
              </main>,
            ]}
          ></Route>
        </ProjectRouter.Switch>
      </div>
    )
  }
}
