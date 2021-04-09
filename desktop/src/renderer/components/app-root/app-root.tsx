import { Component, h } from '@stencil/core'
import { match, Route } from '@stencil/router'
import { Router } from '../../router'

@Component({
  tag: 'app-root',
  styleUrl: 'app-root.css',
  shadow: true,
})
export class AppRoot {
  render() {
    return (
      <main>
        <Router.Switch>
          <Route path="/">
            <app-home></app-home>
          </Route>

          <Route
            path={match('/profile/:name')}
            render={({ name }) => <app-profile name={name}></app-profile>}
          />
        </Router.Switch>
      </main>
    )
  }
}
