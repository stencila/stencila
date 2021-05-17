import { Component, h } from '@stencil/core'
import { Route } from '@stencil/router'
import { initPane } from '../../../store/documentPane/documentPaneActions'
import { getProjectDetails } from '../../../store/project/projectActions'
import { ProjectRouter } from '../projectRouter'

@Component({
  tag: 'app-project-root',
  styleUrl: 'app-project-root.css',
  scoped: true,
})
export class AppProjectRoot {
  componentWillLoad() {
    const projectPath = decodeURI(
      window.location.pathname.replace('/project', '')
    )
    initPane(projectPath)
    getProjectDetails(projectPath)
  }

  render() {
    return (
      <div class="projectWindow">
        <app-project-sidebar-files></app-project-sidebar-files>

        <ProjectRouter.Switch>
          <Route
            path={() => true}
            render={() =>
              <main>
                <app-document-pane></app-document-pane>
              </main>
            }
          ></Route>
        </ProjectRouter.Switch>
      </div>
    )
  }
}
