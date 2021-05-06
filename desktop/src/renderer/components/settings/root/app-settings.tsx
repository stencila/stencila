import { Component, h } from '@stencil/core'
import { Route } from '@stencil/router'
import { SettingsRouter } from '../settingsRouter'

@Component({
  tag: 'app-settings',
  styleUrl: 'app-settings.css',
  scoped: true,
})
export class AppSettings {
  render() {
    return (
      <div class="settingsWindow">
        <app-side-nav></app-side-nav>

        <main>
          <SettingsRouter.Switch>
            <Route path="/settings" to="/settings/general" />

            <Route path="/settings/general">
              <settings-general></settings-general>
            </Route>

            <Route path="/settings/plugins">
              <app-plugins></app-plugins>
            </Route>
          </SettingsRouter.Switch>
        </main>
      </div>
    )
  }
}
