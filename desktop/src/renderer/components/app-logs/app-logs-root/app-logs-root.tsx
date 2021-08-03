import { Component, h } from '@stencil/core'
import { Route } from '@stencil/router'
import { LogsRouter } from '../logsRouter'

@Component({
  tag: 'app-logs-root',
  styleUrl: 'app-logs-root.css',
  scoped: true,
})
export class AppLogs {
  render() {
    return (
      <div class="logsWindow">
        <main>
          <LogsRouter.Switch>
            <Route path="/logs">
              <app-logs-list></app-logs-list>
            </Route>
          </LogsRouter.Switch>
        </main>
      </div>
    )
  }
}
