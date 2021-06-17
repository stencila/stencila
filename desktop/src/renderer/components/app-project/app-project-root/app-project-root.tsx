import { Component, h } from '@stencil/core'
import { Route } from '@stencil/router'
import { store } from '../../../store'
import { initPane } from '../../../store/documentPane/documentPaneActions'
import { fetchProject } from '../../../store/project/projectActions'
import { ProjectRouter } from '../projectRouter'
import { listenForFileEvents, removeFileEventListener } from './projectEvents'

const rootPaneId = 1

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
    initPane(rootPaneId)
    store.dispatch(fetchProject(projectPath))
    listenForFileEvents(projectPath)
  }

  disconnectedCallback() {
    removeFileEventListener()
  }

  render() {
    return (
      <div class="projectWindow">
        <split-me
          n={2}
          sizes={[0.2, 0.8]}
          minSizes={[0.05, 0.2]}
          maxSizes={[0.5, 1]}
          d="horizontal"
        >
          <app-project-sidebar-files slot="0"></app-project-sidebar-files>

          <div slot="1">
            <ProjectRouter.Switch>
              <Route
                path={() => true}
                render={() => (
                  <main>
                    <app-document-pane paneId={rootPaneId}></app-document-pane>
                  </main>
                )}
              ></Route>
            </ProjectRouter.Switch>
          </div>
        </split-me>
      </div>
    )
  }
}
