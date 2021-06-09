import { Component, h } from '@stencil/core'
import { Route } from '@stencil/router'
import { Router } from '../../router'

@Component({
  tag: 'app-root',
  styleUrl: 'app-root.css',
  scoped: true,
})
export class AppRoot {
  render() {
    return (
      <Router.Switch>
        <Route
          path={(path) =>
            path === '/' || path === '/renderer/main_window/index.html'
          }
        >
          <app-launcher></app-launcher>
        </Route>

        <Route path={/^\/settings\/?/}>
          <app-settings-root></app-settings-root>
        </Route>

        <Route path={/^\/onboarding\/?/}>
          <app-onboarding-root></app-onboarding-root>
        </Route>

        <Route path={/^\/project/}>
          <app-project-root></app-project-root>
        </Route>
      </Router.Switch>
    )
  }
}
